use std::any::TypeId;
use crate::any_value::{AnyValue2, AnyValue2Cloneable};

/// Makes AnyValue2Cloneable actually [`Clone`]able.
/// Lazy clone on consumption.
pub struct LazyClone<'a, T: AnyValue2Cloneable>{
    value: &'a T
}

impl<'a, T: AnyValue2Cloneable> LazyClone<'a, T>{
    #[inline]
    pub fn new(value: &'a T) -> Self{
        Self{value}
    }
}

impl<'a, T: AnyValue2Cloneable> AnyValue2 for LazyClone<'a, T>{
    type Type = T::Type;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.value.value_typeid()
    }

    #[inline]
    fn size(&self) -> usize {
        self.value.size()
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.value.bytes()
    }

    #[inline]
    unsafe fn consume_into(self, out: *mut u8) {
        self.value.clone_into(out);
    }
}

impl<'a, T: AnyValue2Cloneable> AnyValue2Cloneable for LazyClone<'a, T>{
    #[inline]
    unsafe fn clone_into(&self, out: *mut u8) {
        self.value.clone_into(out);
    }
}

impl<'a, T: AnyValue2Cloneable> Clone for LazyClone<'a, T>{
    #[inline]
    fn clone(&self) -> Self {
        Self{value: self.value}
    }
}