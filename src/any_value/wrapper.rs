use std::any::TypeId;
use std::mem::size_of;
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
    fn as_bytes_ptr(&self) -> *const u8 {
        &self.value as *const _ as *const u8
    }
}
impl<T: 'static> AnyValuePtrMut for AnyValueWrapper<T> {
    #[inline]
    fn as_bytes_mut_ptr(&mut self) -> *mut u8 {
        &mut self.value as *mut _ as *mut u8
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
impl<T: 'static> AnyValueSizedMut for AnyValueWrapper<T> {}
impl<T: 'static> AnyValueTypedMut for AnyValueWrapper<T> {}