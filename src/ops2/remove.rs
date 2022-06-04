use std::mem::size_of;
use std::marker::PhantomData;
use std::{mem, ptr};
use std::ptr::NonNull;
use crate::{AnyVec, copy_bytes_nonoverlapping};
use crate::any_value::{AnyValue2, AnyValue2Cloneable, Unknown};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops2::temp::Operation;
use crate::traits::Trait;

/// Lazily `swap_remove` element on consumption/drop.
///
/// This `struct` is created by [`AnyVec::swap_remove`].
///
/// [`AnyVec::swap_remove`]: crate::AnyVec::swap_remove
pub struct Remove<'a, Traits: ?Sized + Trait, T: 'static = Unknown>{
    any_vec: &'a mut AnyVec<Traits>,
    index: usize,
    last_index: usize,
    phantom: PhantomData<&'a mut T>
}

impl<'a, Traits: ?Sized + Trait, T: 'static> Remove<'a, Traits, T>{
    #[inline]
    pub(crate) fn new(any_vec: &'a mut AnyVec<Traits>, index: usize) -> Self{
        // 1. mem::forget and element drop panic "safety".
        let last_index = any_vec.raw.len - 1;
        any_vec.raw.len = index;

        Self{any_vec, index, last_index, phantom: PhantomData}
    }
}

impl<'a, Traits: ?Sized + Trait, T: 'static> Operation for Remove<'a, Traits, T>{
    type Traits = Traits;
    type Type = T;

    #[inline]
    fn any_vec(&self) -> &AnyVec<Traits> {
        self.any_vec
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        unsafe{
            if !Unknown::is::<T>(){
                self.any_vec.downcast_ref_unchecked::<T>().as_slice().get_unchecked(self.index)
                    as *const T as *const u8
            } else {
                self.any_vec.raw.mem.as_ptr()
                    .add(self.any_vec.element_layout().size() * self.index)
            }
        }
    }

    #[inline]
    fn consume_op(&mut self) {
    unsafe{
        // 2. shift everything left
        if !Unknown::is::<T>() {
            let dst = self.bytes() as *mut T;
            let src = dst.add(1);
            ptr::copy(src, dst,self.last_index - self.index);
        } else {
            let size = self.any_vec.element_layout().size();
            let dst = self.bytes() as *mut u8;
            let src = dst.add(size);
            ptr::copy(src, dst,size * (self.last_index - self.index));

        }

        // 3. shrink len `self.any_vec.len -= 1`
        self.any_vec.raw.len = self.last_index;
    }
    }
}
