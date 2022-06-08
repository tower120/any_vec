use std::any::TypeId;
use std::ptr::NonNull;
use crate::any_value::AnyValue;
use crate::any_value::Unknown;

/// Non owning byte ptr wrapper.
/// Source should be forgotten.
pub struct AnyValueRaw{
    ptr: NonNull<u8>,
    size: usize,
    typeid: TypeId
}

impl AnyValueRaw{
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, size: usize, typeid: TypeId) -> Self{
        Self{ptr, size, typeid}
    }
}

impl AnyValue for AnyValueRaw{
    type Type = Unknown;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.typeid
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}