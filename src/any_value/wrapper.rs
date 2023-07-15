use std::any::TypeId;
use std::mem::size_of;
use std::ptr::NonNull;
use crate::any_value::{AnyValueTyped, AnyValueTypedMut, AnyValueSizedMut, AnyValueSized, AnyValuePtr, AnyValuePtrMut};

/// Helper struct to convert concrete type to [`AnyValueTypedMut`].
pub struct AnyValueWrapper<T: 'static>{
    value: T
}
impl<T: 'static> AnyValueWrapper<T> {
    #[inline]
    pub fn new(value: T) -> Self{
        Self{ value }
    }
}

impl<T: 'static> AnyValuePtr for AnyValueWrapper<T> {
    type Type = T;

    #[inline]
    fn as_bytes_ptr(&self) -> NonNull<u8> {
        unsafe{NonNull::new_unchecked(
            &self.value as *const _ as *mut u8
        )}
    }
}
impl<T: 'static> AnyValueSized for AnyValueWrapper<T> {
    #[inline]
    fn size(&self) -> usize {
        size_of::<T>()
    }
}
impl<T: 'static> AnyValueTyped for AnyValueWrapper<T> {
    #[inline]
    fn value_typeid(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl<T: 'static> AnyValuePtrMut for AnyValueWrapper<T> {}
impl<T: 'static> AnyValueSizedMut for AnyValueWrapper<T> {}
impl<T: 'static> AnyValueTypedMut for AnyValueWrapper<T> {}