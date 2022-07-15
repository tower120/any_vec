use std::any::TypeId;
use std::ptr::NonNull;
use std::slice;
use crate::any_value::{AnyValue, AnyValueMut, AnyValueMutUnknown, AnyValueUnknown};
use crate::any_value::Unknown;


/// [`AnyValueRaw`] that does not know it's type.
pub struct AnyValueRawUnknown {
    ptr: NonNull<u8>,
    size: usize,
}

impl AnyValueRawUnknown {
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, size: usize) -> Self{
        Self{ptr, size}
    }
}
impl AnyValueUnknown for AnyValueRawUnknown {
    type Type = Unknown;

    #[inline]
    fn as_bytes(&self) -> &[u8]{
        unsafe{slice::from_raw_parts(
            self.ptr.as_ptr(),
            self.size
        )}
    }
}
impl AnyValueMutUnknown for AnyValueRawUnknown {
    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe{slice::from_raw_parts_mut(
            self.ptr.as_ptr(),
            self.size
        )}
    }
}


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
    raw_unsafe: AnyValueRawUnknown,
    typeid: TypeId
}

impl AnyValueRaw{
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, size: usize, typeid: TypeId) -> Self{
        Self{
            raw_unsafe: AnyValueRawUnknown::new(ptr, size),
            typeid
        }
    }
}
impl AnyValueUnknown for AnyValueRaw{
    type Type = Unknown;

    #[inline]
    fn as_bytes(&self) -> &[u8]{
        self.raw_unsafe.as_bytes()
    }
}
impl AnyValue for AnyValueRaw{
    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.typeid
    }
}
impl AnyValueMutUnknown for AnyValueRaw{
    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe{slice::from_raw_parts_mut(
            self.raw_unsafe.ptr.as_ptr(),
            self.raw_unsafe.size
        )}
    }
}
impl AnyValueMut for AnyValueRaw{}