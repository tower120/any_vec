use std::marker::PhantomData;
use std::ptr;
use crate::copy_bytes_nonoverlapping;
use crate::any_value::Unknown;
use crate::any_vec_ptr::IAnyVecRawPtr;
use crate::ops::temp::Operation;

pub struct SwapRemove<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static = Unknown>{
    any_vec_ptr: AnyVecPtr,
    element: *mut u8,
    last_index: usize,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static> SwapRemove<'a, AnyVecPtr, T>{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, index: usize) -> Self{
        let any_vec_raw = unsafe{ any_vec_ptr.any_vec_raw().as_mut() };

        // 1. mem::forget and element drop panic "safety".
        let last_index = any_vec_raw.len - 1;
        any_vec_raw.len = index;

        let element: *mut u8 = unsafe{
            if !Unknown::is::<T>(){
                any_vec_raw.mem.cast::<T>().as_ptr().add(index) as *mut u8
            } else {
                any_vec_raw.mem.as_ptr().add(any_vec_raw.element_layout().size() * index)
            }
        };
        Self{ any_vec_ptr, element, last_index, phantom: PhantomData }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static> Operation for SwapRemove<'a, AnyVecPtr, T>{
    type AnyVecPtr = AnyVecPtr;
    type Type = T;

    #[inline]
    fn any_vec_ptr(&self) -> Self::AnyVecPtr {
        self.any_vec_ptr
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.element
    }

    #[inline]
    fn consume(&mut self) {
    unsafe{
        // 2. overwrite with last element
        let any_vec_raw = self.any_vec_ptr.any_vec_raw().as_mut();
        let last_element =
            if !Unknown::is::<T>() {
                any_vec_raw.mem.cast::<T>().as_ptr().add(self.last_index) as *const u8
            } else {
                any_vec_raw.mem.as_ptr()
                    .add(any_vec_raw.element_layout().size() * self.last_index)
            };

        if self.element as *const u8 != last_element {
            if !Unknown::is::<T>() {
                ptr::copy_nonoverlapping
                    (last_element as *const T, self.element as *mut T, 1);
            } else {
                copy_bytes_nonoverlapping
                    (last_element, self.element, any_vec_raw.element_layout().size());
            }
        }

        // 3. shrink len `self.any_vec.len -= 1`
        any_vec_raw.len = self.last_index;
    }
    }
}
