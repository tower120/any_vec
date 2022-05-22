use std::{mem, ptr};
use std::alloc::{alloc, dealloc, Layout, realloc, handle_alloc_error};
use std::any::TypeId;
use std::marker::PhantomData;
use std::ptr::{NonNull, null_mut};
use crate::{AnyVecMut, AnyVecRef, AnyVecTyped, copy_bytes_nonoverlapping, Unknown};
use crate::any_value::{AnyValue};
use crate::any_value::AnyValueTemp;
use crate::ops::{SwapRemove};

/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// *`Element: 'static` due to TypeId requirements*
pub struct AnyVec {
    pub(crate) mem: NonNull<u8>,
    capacity: usize,        // in elements
    pub(crate) len: usize,             // in elements
    element_layout: Layout, // size is aligned
    type_id: TypeId,        // purely for safety checks
    pub(crate) drop_fn: Option<fn(ptr: *mut u8, len: usize)>
}

impl AnyVec {
    pub fn new<Element: 'static>() -> Self {
        Self::with_capacity::<Element>(0)
    }

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
                NonNull::new_unchecked(self as *const _ as *mut _)
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
            any_vec_typed: AnyVecTyped::new(NonNull::new_unchecked(self))
        }
    }

    /// This is the only function, which do allocations/deallocations.
    /// Real capacity one element bigger. Last virtual element used by remove operations,
    /// as temporary value location.
    fn set_capacity(&mut self, new_capacity: usize){
        // Never cut
        debug_assert!(self.len <= new_capacity);

        if self.capacity == new_capacity {
            return;
        }

        if self.element_layout.size() != 0 {
            unsafe{
                // Non checked mul, because this memory size already allocated.
                // +1 - from temporary element storage
                let mem_layout = Layout::from_size_align_unchecked(
                    self.element_layout.size() * (self.capacity + 1),
                    self.element_layout.align()
                );

                self.mem =
                    if new_capacity == 0 {
                        dealloc(self.mem.as_ptr(), mem_layout);
                        NonNull::<u8>::dangling()
                    } else {
                        // mul carefully, to prevent overflow.
                        // +1 - for temporary element storage
                        let new_mem_size = self.element_layout.size()
                            .checked_mul(new_capacity + 1).unwrap();
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
    fn index_check(&self, index: usize){
        assert!(index < self.len, "Index out of range!");
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self){
        self.set_capacity(
             if self.capacity == 0 {2} else {self.capacity * 2}
        );
    }

    /// # Panics
    ///
    /// * Panics if index is out of bounds.
    /// * Panics if type mismatch.
    /// * Panics if out of memory.
    pub fn insert<V: AnyValue>(&mut self, index: usize, value: V) {
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
                value.consume_bytes(|value_bytes|{
                    ptr::copy_nonoverlapping(
                        value_bytes.cast::<V::Type>().as_ptr(),
                        element,
                        1
                    );
                });
            } else {
                let element_size = self.element_layout.size();
                let element = self.mem.as_ptr().add(element_size * index);

                // push right
                ptr::copy
                (
                    element,
                    element.add(element_size),
                    element_size * (self.len - index)
                );

                value.consume_bytes(|value_bytes|{
                    copy_bytes_nonoverlapping(
                        value_bytes.as_ptr(),
                        element,
                        element_size
                    );
                });
            }
        }

        self.len += 1;
    }

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if out of memory.
    #[inline]
    pub fn push<V: AnyValue>(&mut self, value: V) {
        assert_eq!(value.value_typeid(), self.type_id);
        if self.len == self.capacity{
            self.grow();
        }

        unsafe{
            // Compile time type optimization
            if !Unknown::is::<V::Type>(){
                value.consume_bytes(|value_bytes|{
                    ptr::copy_nonoverlapping(
                        value_bytes.cast::<V::Type>().as_ptr(),
                        self.mem.cast::<V::Type>().as_ptr().add(self.len),
                        1
                    );
                });
            } else {
                let element_size = self.element_layout.size();
                let new_element = self.mem.as_ptr().add(element_size * self.len);
                value.consume_bytes(|value_bytes|{
                    copy_bytes_nonoverlapping(
                        value_bytes.as_ptr(),
                        new_element,
                        element_size
                    );
                });
            }
        }

        self.len += 1;
    }

    #[inline]
    fn drop_element(&mut self, ptr: *mut u8, len: usize){
        if let Some(drop_fn) = self.drop_fn{
            (drop_fn)(ptr, len);
        }
    }

    /// element_size as parameter - because it possible can be known at compile time
    #[inline]
    pub(crate) unsafe fn remove_into_impl(&mut self, index: usize, element_size: usize, out: *mut u8) {
        self.index_check(index);

        let element = self.mem.as_ptr().add(element_size * index);

        // 1. copy element to out
        if !out.is_null() {
            copy_bytes_nonoverlapping(element, out, element_size);
        }

        // 2. shift everything left
        ptr::copy(
            element.add(element_size),
            element,
            element_size * (self.len - index - 1)
        );

        // 3. shrink len
        self.len -= 1;
    }

    /// Type erased version of [`Vec::remove`]. Due to this, does not return element.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn remove(&mut self, index: usize) {
    unsafe{
        let temp_element = if self.drop_fn.is_some() {
            self.mem.as_ptr().add(self.element_layout.size() * self.capacity)
        } else {
            null_mut()
        };

        self.remove_into_impl(index, self.element_layout.size(), temp_element);

        // drop element in temporary storage
        if !temp_element.is_null() {
            (self.drop_fn.unwrap_unchecked())(temp_element, 1);
        }
    }
    }

    /// Same as [`remove`], but copy removed element as bytes to `out`.
    ///
    /// [`remove`]: Self::remove
    ///
    /// # Safety
    /// * It is your responsibility to properly drop `out` element.
    ///
    /// # Panics
    /// * Panics if index out of bounds.
    /// * Panics if out len does not match element size.
    #[inline]
    pub unsafe fn remove_into(&mut self, index: usize, out: &mut[u8]) {
        assert_eq!(out.len(), self.element_layout.size());
        self.remove_into_impl(index, self.element_layout.size(), out.as_mut_ptr());
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> AnyValueTemp<SwapRemove> {
        self.index_check(index);

        AnyValueTemp(SwapRemove {
            any_vec: self,
            index,
            phantom: PhantomData
        })
    }

    #[inline]
    pub fn clear(&mut self){
        let len = self.len;

        // Prematurely set the length to zero so that even if dropping the values panics users
        // won't be able to access the dropped values.
        self.len = 0;

        self.drop_element(self.mem.as_ptr(), len);
    }

    #[inline]
    pub(crate) unsafe fn as_slice_unchecked<T>(&self) -> &[T]{
        std::slice::from_raw_parts(
            self.mem.as_ptr().cast::<T>(),
            self.len,
        )
    }

    #[inline]
    pub(crate) unsafe fn as_mut_slice_unchecked<T>(&mut self) -> &mut[T]{
        std::slice::from_raw_parts_mut(
            self.mem.as_ptr().cast::<T>(),
            self.len,
        )
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
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for AnyVec {
    fn drop(&mut self) {
        self.clear();
        self.set_capacity(0);
    }
}
