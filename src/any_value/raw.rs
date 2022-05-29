use std::any::TypeId;
use std::ptr::NonNull;
use crate::any_value::AnyValue;
use crate::any_value::Unknown;

/// Non owning byte ptr wrapper.
/// Source should be forgotten.
pub struct AnyValueRaw{
    ptr: NonNull<u8>,
    /*size: usize,*/
    typeid: TypeId
}

impl AnyValueRaw{
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, /*len: usize, */typeid: TypeId) -> Self{
        Self{ptr, /*size: len,*/ typeid}
    }

/*    #[inline]
    pub fn value_size(&self) -> usize{
        self.size
    }*/
}

impl AnyValue for AnyValueRaw{
    type Type = Unknown;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.typeid
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F) {
        f(self.ptr);
    }
}