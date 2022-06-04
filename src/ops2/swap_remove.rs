use std::any::TypeId;
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
pub struct SwapRemove<'a, Traits: ?Sized + Trait, T: 'static = Unknown>{
    any_vec: &'a mut AnyVec<Traits>,
    element: *mut u8,
    last_index: usize,
    phantom: PhantomData<&'a mut T>
}

impl<'a, Traits: ?Sized + Trait, T: 'static> SwapRemove<'a, Traits, T>{
    #[inline]
    pub(crate) fn new(any_vec: &'a mut AnyVec<Traits>, index: usize) -> Self{
        // 1. mem::forget and element drop panic "safety".
        let last_index = any_vec.raw.len - 1;
        any_vec.raw.len = index;

        let element: *mut u8 = unsafe{
            if !Unknown::is::<T>(){
                any_vec.downcast_mut_unchecked::<T>().as_mut_slice().get_unchecked_mut(index)
                    as *mut T as *mut u8
            } else {
                any_vec.raw.mem.as_ptr().add(any_vec.element_layout().size() * index)
            }
        };
        Self{any_vec, element, last_index, phantom: PhantomData}
    }
}

impl<'a, Traits: ?Sized + Trait, T: 'static> Operation for SwapRemove<'a, Traits, T>{
    type Traits = Traits;
    type Type = T;

    #[inline]
    fn any_vec(&self) -> &AnyVec<Traits> {
        self.any_vec
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.element
    }

    #[inline]
    fn consume_op(&mut self) {
        // 2. overwrite with last element
        unsafe{
            let last_element =
                if !Unknown::is::<T>() {
                    self.any_vec.downcast_mut_unchecked::<T>().as_mut_slice()
                        .get_unchecked_mut(self.last_index) as *mut T as *mut u8
                } else {
                    self.any_vec.raw.mem.as_ptr()
                        .add(self.any_vec.element_layout().size() * self.last_index)
                };

            if self.element != last_element {
                if !Unknown::is::<T>() {
                    ptr::copy_nonoverlapping
                        (last_element as *const T, self.element as *mut T, 1);
                } else {
                    copy_bytes_nonoverlapping
                        (last_element, self.element, self.any_vec.element_layout().size());
                }
            }
        }

        // 3. shrink len `self.any_vec.len -= 1`
        self.any_vec.raw.len = self.last_index;
    }
}
