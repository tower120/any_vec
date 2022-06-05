use std::{mem, ptr};
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout, realloc};
use std::any::TypeId;
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::{AnyVecMut, AnyVecRef, AnyVecTyped};
use crate::any_value::{AnyValue, Unknown};
use crate::clone_type::CloneFn;

pub type DropFn = fn(ptr: *mut u8, len: usize);

pub struct AnyVecRaw {
    pub(crate) mem: NonNull<u8>,
    capacity: usize,        // in elements
    pub(crate) len: usize,  // in elements
    element_layout: Layout, // size is aligned
    type_id: TypeId,        // purely for safety checks
    drop_fn: Option<DropFn>
}

impl AnyVecRaw {
/*    pub fn new<Element: 'static>() -> Self {
        Self::with_capacity::<Element>(0)
    }
*/
    pub fn with_capacity<Element: 'static>(capacity: usize) -> Self {
        let mut this = Self{
            mem: NonNull::<u8>::dangling(),
            capacity: 0,
            len: 0,
            element_layout: Layout::new::<Element>(),
            type_id: TypeId::of::<Element>(),
            drop_fn:
                if !mem::needs_drop::<Element>(){
                    None
                } else{
                    Some(|mut ptr: *mut u8, len: usize|{
                        for _ in 0..len{
                            unsafe{
                                ptr::drop_in_place(ptr as *mut Element);
                                ptr = ptr.add(mem::size_of::<Element>());
                            }
                        }
                    })
                }
        };
        this.set_capacity(capacity);
        this
    }

    #[inline]
    pub fn drop_fn(&self) -> Option<DropFn>{
        self.drop_fn
    }

    /// Unsafe, because type cloneability is not checked
    pub(crate) unsafe fn clone(&self, clone_fn: Option<CloneFn>) -> Self {
        // 1. construct empty "prototype"
        let mut cloned = Self{
            mem: NonNull::<u8>::dangling(),
            capacity: 0,
            len: 0,
            element_layout: self.element_layout,
            type_id: self.type_id,
            drop_fn: self.drop_fn,
            //clone_fn: self.clone_fn
        };

        // 2. allocate
        // TODO: set only necessary capacity size.
        // TODO: implement through expand.
        cloned.set_capacity(self.capacity);

        // 3. copy/clone
        {
            let src = self.mem.as_ptr();
            let dst = cloned.mem.as_ptr();
            if let Some(clone_fn) = clone_fn{
                (clone_fn)(src, dst, self.len);
            } else {
                ptr::copy_nonoverlapping(
                    src, dst,
                    self.element_layout.size() * self.len
                );
            }
        }

        // 4. set len
        cloned.len = self.len;
        cloned
    }

    #[inline]
    pub fn downcast_ref<Element: 'static>(&self) -> Option<AnyVecRef<Element>> {
        if self.type_id == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<Element: 'static>(&self) -> AnyVecRef<Element> {
        AnyVecRef{
            any_vec_typed: (AnyVecTyped::new(
                NonNull::from(self)
            ))
        }
    }

    #[inline]
    pub fn downcast_mut<Element: 'static>(&mut self) -> Option<AnyVecMut<Element>> {
        if self.type_id == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<Element: 'static>(&mut self) -> AnyVecMut<Element> {
        AnyVecMut{
            any_vec_typed: AnyVecTyped::new(NonNull::from(self))
        }
    }

    /// This is the only function, which do allocations/deallocations.
    fn set_capacity(&mut self, new_capacity: usize){
        // Never cut
        debug_assert!(self.len <= new_capacity);

        if self.capacity == new_capacity {
            return;
        }

        if self.element_layout.size() != 0 {
            unsafe{
                // Non checked mul, because this memory size already allocated.
                let mem_layout = Layout::from_size_align_unchecked(
                    self.element_layout.size() * self.capacity,
                    self.element_layout.align()
                );

                self.mem =
                    if new_capacity == 0 {
                        dealloc(self.mem.as_ptr(), mem_layout);
                        NonNull::<u8>::dangling()
                    } else {
                        // mul carefully, to prevent overflow.
                        let new_mem_size = self.element_layout.size()
                            .checked_mul(new_capacity).unwrap();
                        let new_mem_layout = Layout::from_size_align_unchecked(
                            new_mem_size, self.element_layout.align()
                        );

                        if self.capacity == 0 {
                            // allocate
                            NonNull::new(alloc(new_mem_layout))
                        } else {
                            // reallocate
                            NonNull::new(realloc(
                                self.mem.as_ptr(), mem_layout,new_mem_size
                            ))
                        }
                        .unwrap_or_else(|| handle_alloc_error(new_mem_layout))
                    }
            }
        }
        self.capacity = new_capacity;
    }

    #[inline]
    pub(crate) fn index_check(&self, index: usize){
        assert!(index < self.len, "Index out of range!");
    }

    #[inline]
    fn type_check<V: AnyValue>(&self, value: &V){
        assert_eq!(value.value_typeid(), self.type_id, "Type mismatch!");
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self){
        self.set_capacity(
             if self.capacity == 0 {2} else {self.capacity * 2}
        );
    }

    pub fn insert<V: AnyValue>(&mut self, index: usize, value: V) {
        self.type_check(&value);
        assert!(index <= self.len, "Index out of range!");

        if self.len == self.capacity{
            self.grow();
        }

        unsafe{
            // Compile time type optimization
            if !Unknown::is::<V::Type>(){
                let element = self.mem.cast::<V::Type>().as_ptr().add(index);

                // 1. shift right
                ptr::copy(
                    element,
                    element.add(1),
                    self.len - index
                );

                // 2. write value
                value.move_into(element as *mut u8);
            } else {
                let element_size = self.element_layout.size();
                let element = self.mem.as_ptr().add(element_size * index);

                // 1. shift right
                ptr::copy(
                    element,
                    element.add(element_size),
                    element_size * (self.len - index)
                );

                // 2. write value
                value.move_into(element);
            }
        }

        self.len += 1;
    }

    #[inline]
    pub fn push<V: AnyValue>(&mut self, value: V) {
        self.type_check(&value);

        if self.len == self.capacity{
            self.grow();
        }

        unsafe{
            // Compile time type optimization
            let element =
                if !Unknown::is::<V::Type>(){
                     self.mem.cast::<V::Type>().as_ptr().add(self.len) as *mut u8
                } else {
                    let element_size = self.element_layout.size();
                    self.mem.as_ptr().add(element_size * self.len)
                };

            value.move_into(element);
        }

        self.len += 1;
    }

    #[inline]
    pub fn clear(&mut self){
        let len = self.len;

        // Prematurely set the length to zero so that even if dropping the values panics users
        // won't be able to access the dropped values.
        self.len = 0;

        if let Some(drop_fn) = self.drop_fn{
            (drop_fn)(self.mem.as_ptr(), len);
        }
    }

    /// Element TypeId
    #[inline]
    pub fn element_typeid(&self) -> TypeId{
        self.type_id
    }

    /// Element Layout
    #[inline]
    pub fn element_layout(&self) -> Layout {
        self.element_layout
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for AnyVecRaw {
    fn drop(&mut self) {
        self.clear();
        self.set_capacity(0);
    }
}
