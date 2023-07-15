use std::any::TypeId;
use std::ptr::NonNull;
use crate::any_value::{AnyValueTyped, AnyValueTypedMut, AnyValueSizedMut, AnyValueSized, AnyValuePtr, AnyValuePtrMut};
use crate::any_value::Unknown;

/// Non owning byte ptr wrapper for feeding AnyVec.
///
/// Source should be forgotten, before pushing to AnyVec.
/// Contained value **WILL NOT** be dropped on AnyValueRawPtr drop.
///
/// This is useful to fill [AnyVec] directly from raw bytes,
/// without intermediate casting to concrete type.
///
/// # Example
/// ```rust
/// # use std::any::TypeId;
/// # use std::mem;
/// # use std::mem::size_of;
/// # use std::ptr::NonNull;
/// # use any_vec::AnyVec;
/// # use any_vec::any_value::AnyValueRawPtr;
/// let s = String::from("Hello!");
/// let raw_value = unsafe{AnyValueRawPtr::new(
///     NonNull::from(&s).cast::<u8>()
/// )};
/// mem::forget(s);
///
/// let mut any_vec: AnyVec = AnyVec::new::<String>();
/// unsafe{
///     any_vec.push_unchecked(raw_value);
/// }
/// ```
///
/// [AnyVec]: crate::AnyVec
pub struct AnyValueRawPtr {
    ptr: NonNull<u8>
}

impl AnyValueRawPtr {
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>) -> Self{
        Self{ptr}
    }
}
impl AnyValuePtr for AnyValueRawPtr {
    type Type = Unknown;

    #[inline]
    fn as_bytes_ptr(&self) -> NonNull<u8> {
        unsafe{
            NonNull::from(self.ptr.as_ref())
        }
    }
}
impl AnyValuePtrMut for AnyValueRawPtr {}


/// [AnyValueRawPtr] that know it's size.
pub struct AnyValueRawSized {
    ptr: NonNull<u8>,
    size: usize,
}

impl AnyValueRawSized {
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, size: usize) -> Self{
        Self{ptr, size}
    }
}

impl AnyValuePtr for AnyValueRawSized {
    type Type = Unknown;

    #[inline]
    fn as_bytes_ptr(&self) -> NonNull<u8> {
        unsafe{
            NonNull::from(self.ptr.as_ref())
        }
    }
}
impl AnyValueSized for AnyValueRawSized {
    #[inline]
    fn size(&self) -> usize {
        self.size
    }
}

impl AnyValuePtrMut for AnyValueRawSized {}
impl AnyValueSizedMut for AnyValueRawSized {}


/// [AnyValueRawSized] that know it's type.
pub struct AnyValueRawTyped {
    raw_unsafe: AnyValueRawSized,
    typeid: TypeId
}

impl AnyValueRawTyped {
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, size: usize, typeid: TypeId) -> Self{
        Self{
            raw_unsafe: AnyValueRawSized::new(ptr, size),
            typeid
        }
    }
}

impl AnyValuePtr for AnyValueRawTyped {
    type Type = Unknown;

    #[inline]
    fn as_bytes_ptr(&self) -> NonNull<u8> {
        self.raw_unsafe.ptr
    }
}
impl AnyValueSized for AnyValueRawTyped {
    #[inline]
    fn size(&self) -> usize {
        self.raw_unsafe.size
    }
}
impl AnyValueTyped for AnyValueRawTyped {
    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.typeid
    }
}

impl AnyValuePtrMut   for AnyValueRawTyped {}
impl AnyValueSizedMut for AnyValueRawTyped {}
impl AnyValueTypedMut for AnyValueRawTyped {}