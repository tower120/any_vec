use std::mem::size_of;
use std::marker::PhantomData;
use std::ptr;
use std::ptr::NonNull;
use crate::{AnyVec, copy_bytes_nonoverlapping, Unknown};
use crate::any_value::temp::Impl;

/// Lazily `swap_remove` element on consumption/drop.
pub struct SwapRemove<'a, T: 'static = Unknown>{
    pub(crate) any_vec: &'a mut AnyVec,
    pub(crate) index: usize,
    pub(crate) phantom: PhantomData<&'a mut T>
}

impl<'a, T: 'static> Impl for SwapRemove<'a, T>{
    type Type = T;

    #[inline]
    fn any_vec(&self) -> &AnyVec {
        self.any_vec
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(&mut self, f: F) {
        // TODO: as separate fn
        let element_size = if Unknown::is::<T>() {
            self.any_vec.element_layout().size()
        } else {
            size_of::<T>()
        };
        let element = self.any_vec.mem.as_ptr().add(element_size * self.index);

        // mem::forget and element drop panic "safety".
        let last_index = self.any_vec.len - 1;
        self.any_vec.len = self.index;

        // 1. Consume
        f(NonNull::new_unchecked(element));

        // 2. overwrite with last element
        let last_element = self.any_vec.mem.as_ptr().add(element_size * last_index);
        if self.index != last_index {
            if Unknown::is::<T>() {
                copy_bytes_nonoverlapping(last_element, element, element_size);
            } else {
                ptr::copy_nonoverlapping(last_element as *const T, element as *mut T, 1);
            }
        }

        // 3. shrink len `self.any_vec.len -= 1`
        self.any_vec.len = last_index
    }
}
