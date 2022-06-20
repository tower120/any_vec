use std::{cmp, mem, ptr};
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout, realloc};
use std::any::TypeId;
use std::ptr::NonNull;
use crate::any_value::{AnyValue, Unknown};
use crate::clone_type::CloneFn;
use crate::mem::Mem;

pub type DropFn = fn(ptr: *mut u8, len: usize);

pub struct AnyVecRaw<M: Mem> {
    pub(crate) mem: M,
    pub(crate) len: usize,  // in elements
    type_id: TypeId,        // purely for safety checks
    drop_fn: Option<DropFn>
}

impl<M: Mem> AnyVecRaw<M> {
    #[inline]
    pub fn with_capacity<Element: 'static>(capacity: usize) -> Self {
        Self{
            mem: M::new(Layout::new::<Element>(), capacity),
            len: 0,
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
        }
    }

    #[inline]
    pub fn drop_fn(&self) -> Option<DropFn>{
        self.drop_fn
    }

    #[inline]
    pub(crate) fn clone_empty(&self) -> Self{
        Self{
            mem: M::new(self.element_layout(), 0),
            len: 0,
            type_id: self.type_id,
            drop_fn: self.drop_fn,
        }
    }

    /// Unsafe, because type cloneability is not checked
    pub(crate) unsafe fn clone(&self, clone_fn: Option<CloneFn>) -> Self {
        // 1. construct empty "prototype"
        let mut cloned = self.clone_empty();

        // 2. allocate
        cloned.mem.expand(self.len);

        // 3. copy/clone
        {
            let src = self.mem.as_ptr();
            let dst = cloned.mem.as_mut_ptr();
            if let Some(clone_fn) = clone_fn{
                (clone_fn)(src, dst, self.len);
            } else {
                ptr::copy_nonoverlapping(
                    src, dst,
                    self.element_layout().size() * self.len
                );
            }
        }

        // 4. set len
        cloned.len = self.len;
        cloned
    }

    #[inline]
    pub(crate) fn index_check(&self, index: usize){
        assert!(index < self.len, "Index out of range!");
    }

    #[inline]
    pub(crate) fn type_check<V: AnyValue>(&self, value: &V){
        assert_eq!(value.value_typeid(), self.type_id, "Type mismatch!");
    }

    #[cold]
    #[inline(never)]
    fn expand_one(&mut self){
        self.mem.expand(1);
    }

    #[inline]
    /*pub(crate)*/ fn reserve_one(&mut self){
        if self.len == self.capacity(){
            self.expand_one();
        }
    }

    // TODO: UNTESTED!!!
    #[inline]
    pub fn reserve(&mut self, additional: usize){
        let new_len = self.len + additional;
        if self.capacity() < new_len{
            self.mem.expand(new_len - self.capacity());
        }
    }

    // TODO: UNTESTED!!!
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize){
        let new_len = self.len + additional;
        if self.capacity() < new_len{
            self.mem.expand_exact(new_len - self.capacity());
        }
    }

    // TODO: UNTESTED!!!
    pub fn shrink_to_fit(&mut self){
        self.mem.resize(self.len);
    }

    // TODO: UNTESTED!!!
    pub fn shrink_to(&mut self, min_capacity: usize){
        let new_len = cmp::max(self.len, min_capacity);
        self.mem.resize(new_len);
    }

    /// # Safety
    ///
    /// Type is not checked.
    pub unsafe fn insert_unchecked<V: AnyValue>(&mut self, index: usize, value: V) {
        assert!(index <= self.len, "Index out of range!");

        self.reserve_one();

        // Compile time type optimization
        if !Unknown::is::<V::Type>(){
            let element = self.mem.as_mut_ptr().cast::<V::Type>().add(index);

            // 1. shift right
            ptr::copy(
                element,
                element.add(1),
                self.len - index
            );

            // 2. write value
            value.move_into(element as *mut u8);
        } else {
            let element_size = self.element_layout().size();
            let element = self.mem.as_mut_ptr().add(element_size * index);

            // 1. shift right
            crate::copy_bytes(
                element,
                element.add(element_size),
                element_size * (self.len - index)
            );

            // 2. write value
            value.move_into(element);
        }

        self.len += 1;
    }

    /// # Safety
    ///
    /// Type is not checked.
    #[inline]
    pub unsafe fn push_unchecked<V: AnyValue>(&mut self, value: V) {
        self.reserve_one();

        // Compile time type optimization
        let element =
            if !Unknown::is::<V::Type>(){
                 self.mem.as_mut_ptr().cast::<V::Type>().add(self.len) as *mut u8
            } else {
                let element_size = self.element_layout().size();
                self.mem.as_mut_ptr().add(element_size * self.len)
            };

        value.move_into(element);

        self.len += 1;
    }

    #[inline]
    pub fn clear(&mut self){
        let len = self.len;

        // Prematurely set the length to zero so that even if dropping the values panics users
        // won't be able to access the dropped values.
        self.len = 0;

        if let Some(drop_fn) = self.drop_fn{
            (drop_fn)(self.mem.as_mut_ptr(), len);
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
        self.mem.element_layout()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.mem.size()
    }
}

impl<M: Mem> Drop for AnyVecRaw<M> {
    fn drop(&mut self) {
        self.clear();
    }
}
