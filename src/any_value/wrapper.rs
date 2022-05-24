use std::any::TypeId;
use std::mem;
use std::mem::ManuallyDrop;
use std::ptr;
use std::ptr::NonNull;
use crate::any_value::AnyValue;

/// Helper struct to convert concrete type to [`AnyValue`]
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
    fn downcast<U: 'static>(self) -> U {
        assert_eq!(self.value_typeid(), TypeId::of::<U>());
        // rust don't see that types are the same after assert.
        let value = ManuallyDrop::new(self.value);
        let ptr = &*value as *const T as *const U;
        unsafe { ptr::read(ptr) }
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(mut self, f: F) {
        f(NonNull::new_unchecked(&mut self.value as *mut _  as *mut u8));
        mem::forget(self.value);
    }
}
