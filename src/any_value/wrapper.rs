use std::any::TypeId;
use std::mem::{ManuallyDrop, size_of};
use std::ptr;
use crate::any_value::AnyValue;

/// Helper struct to convert concrete type to [`AnyValue`].
pub struct AnyValueWrapper<T: 'static>{
    value: T
}
impl<T: 'static> AnyValueWrapper<T> {
    #[inline]
    pub fn new(value: T) -> Self{
        Self{ value }
    }
}
impl<T: 'static> AnyValue for AnyValueWrapper<T> {
    type Type = T;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline]
    fn size(&self) -> usize {
        size_of::<T>()
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        &self.value as *const _ as *const u8
    }

    #[inline]
    unsafe fn downcast_unchecked<U: 'static>(self) -> U {
        // rust don't see that types are the same after assert.
        let value = ManuallyDrop::new(self.value);
        let ptr = &*value as *const T as *const U;
        ptr::read(ptr)
    }
}
