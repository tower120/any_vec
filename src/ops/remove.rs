use std::marker::PhantomData;
use std::ptr;
use crate::any_value::Unknown;
use crate::any_vec_ptr::IAnyVecRawPtr;
use crate::any_vec_raw::AnyVecRaw;
use crate::mem::Mem;
use crate::ops::temp::Operation;

pub struct Remove<'a, AnyVecPtr: IAnyVecRawPtr>{
    any_vec_ptr: AnyVecPtr,
    index: usize,
    last_index: usize,
    phantom: PhantomData<&'a mut AnyVecRaw<AnyVecPtr::M>>
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> Remove<'a, AnyVecPtr>{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, index: usize) -> Self{
        // 1. mem::forget and element drop panic "safety".
        let any_vec_raw = unsafe{ any_vec_ptr.any_vec_raw().as_mut() };
        let last_index = any_vec_raw.len - 1;
        any_vec_raw.len = index;

        Self{any_vec_ptr, index, last_index, phantom: PhantomData}
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> Operation for Remove<'a, AnyVecPtr>{
    type AnyVecPtr = AnyVecPtr;

    #[inline]
    fn any_vec_ptr(&self) -> Self::AnyVecPtr {
        self.any_vec_ptr
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        unsafe{
            let any_vec_raw = self.any_vec_ptr.any_vec_raw().as_ref();
            if !Unknown::is::<AnyVecPtr::Element>(){
                any_vec_raw.mem.as_ptr().cast::<AnyVecPtr::Element>()
                    .add(self.index) as *const u8
            } else {
                any_vec_raw.mem.as_ptr()
                    .add(any_vec_raw.element_layout().size() * self.index)
            }
        }
    }

    #[inline]
    fn consume(&mut self) {
    unsafe{
        // 2. shift everything left
        if !Unknown::is::<AnyVecPtr::Element>() {
            let dst = self.bytes() as *mut AnyVecPtr::Element;
            let src = dst.add(1);
            ptr::copy(src, dst,self.last_index - self.index);
        } else {
            let size = self.any_vec_ptr.any_vec_raw().as_ref().element_layout().size();
            let dst = self.bytes() as *mut u8;
            let src = dst.add(size);
            crate::copy_bytes(src, dst,size * (self.last_index - self.index));
        }

        // 3. shrink len `self.any_vec.len -= 1`
        {
            let any_vec_raw = self.any_vec_ptr.any_vec_raw().as_mut();
            any_vec_raw.len = self.last_index;
        }
    }
    }
}
