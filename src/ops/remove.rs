use std::any::{Any, TypeId};
use std::mem::size_of;
use std::marker::PhantomData;
use std::mem::{forget, MaybeUninit};
use std::ptr;
use std::ptr::NonNull;
use crate::{AnyVec, copy_bytes_nonoverlapping, Unknown};
use crate::any_value::temp::Impl;

/// Lazily `remove` element on consumption/drop.
pub struct Remove<'a, T: 'static = Unknown>{
    pub(crate) any_vec: &'a mut AnyVec,
    pub(crate) index: usize,
    pub(crate) phantom: PhantomData<&'a mut T>
}

impl<'a, T: 'static> Impl for Remove<'a, T>{
    type Type = T;

    #[inline]
    fn any_vec(&self) -> &AnyVec {
        self.any_vec
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(&mut self, f: F) {
        // Unknown type
        // ----------------------------
        let element_size = self.any_vec.element_layout().size();
        let element = self.any_vec.mem.as_ptr().add(element_size * self.index);

        // mem::forget and element drop panic "safety".
        let last_index = self.any_vec.len - 1;
        self.any_vec.len = self.index;

        // 1. consume
        f(NonNull::new_unchecked(element));

        // 2. shift everything left
        ptr::copy(
            element.add(element_size),
            element,
            element_size * (last_index - self.index)  // self.any_vec.len - self.index - 1
        );

        // 3. shrink len `self.any_vec.len -= 1`
        self.any_vec.len = last_index;
    }
}
