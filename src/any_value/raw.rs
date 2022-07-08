use std::any::TypeId;
use std::ptr::NonNull;
use std::slice;
use crate::any_value::AnyValue;
use crate::any_value::Unknown;

/// Non owning byte ptr wrapper.
/// Source should be forgotten.
///
/// # Example
/// ```rust
/// # use std::any::TypeId;
/// # use std::mem;
/// # use std::mem::size_of;
/// # use std::ptr::NonNull;
/// # use any_vec::AnyVec;
/// # use any_vec::any_value::AnyValueRaw;
/// let s = String::from("Hello!");
/// let raw_value = unsafe{AnyValueRaw::new(
///     NonNull::from(&s).cast::<u8>(),
///     size_of::<String>(),
///     TypeId::of::<String>()
/// )};
/// mem::forget(s);
///
/// let mut any_vec: AnyVec = AnyVec::new::<String>();
/// any_vec.push(raw_value);
/// ```
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
    fn as_bytes(&self) -> &[u8]{
        unsafe{slice::from_raw_parts(
            self.ptr.as_ptr(),
            self.size
        )}
    }
}