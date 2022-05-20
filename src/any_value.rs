use std::any::{Any, TypeId};
use std::{mem, ptr};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, size_of};
use std::ops::DerefMut;
use std::ptr::{drop_in_place, NonNull, null_mut};

pub trait AnyValue {
    /// Known type size.
    /// Used for optimization.
    const KNOWN_SIZE: Option<usize> = None;

    fn value_typeid(&self) -> TypeId;

    /// # Panic
    ///
    /// Panics if type mismatch
    fn downcast<T: 'static>(self) -> T;

    /// Consume value as bytes.
    /// It is your responsibility to properly drop it.
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F);
}

/// Helper struct to convert concrete type to [`IAnyValue`]
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
    const KNOWN_SIZE: Option<usize> = Some(size_of::<T>());

    #[inline]
    fn value_typeid(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline]
    fn downcast<U: 'static>(self) -> U {
        assert_eq!(self.value_typeid(), TypeId::of::<U>());
        // rust don't see that types are the same after assert.
        unsafe {
            let self_T = &self.value as *const T;
            let self_U = self_T as *const U;
            mem::forget(self.value);
            ptr::read(self_U)
        }
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(mut self, f: F) {
        f(NonNull::new_unchecked(&mut self.value as *mut _  as *mut u8));
        mem::forget(self.value);
    }
}

// AnyValueTemp - temporary existing value in memory, data will be erased with AnyValueTemp destruction.
// May do some postponed actions on consumption/destruction.
pub struct AnyValueTemp<'a, DropFn: FnOnce(*mut u8)>{
    mem: NonNull<u8>,
    typeid: TypeId,
    drop_fn: ManuallyDrop<DropFn>,
    phantom: PhantomData<&'a mut [u8]>
}

impl<'a, DropFn: FnOnce(*mut u8)> AnyValueTemp<'a, DropFn>{
    #[inline]
    pub unsafe fn from_raw_parts(mem: NonNull<u8>, typeid: TypeId, drop_fn: DropFn) -> Self {
        Self{
            mem,
            typeid,
            drop_fn: ManuallyDrop::new(drop_fn),
            phantom: PhantomData
        }
    }
}

impl<'a, DropFn: FnOnce(*mut u8)> AnyValue for AnyValueTemp<'a, DropFn>{
    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.typeid
    }

    #[inline]
    fn downcast<T: 'static>(self) -> T{
        assert_eq!(self.typeid, TypeId::of::<T>());
        unsafe{
            let value = ptr::read(self.mem.as_ptr().cast::<T>());
            let drop_fn = ptr::read(&*self.drop_fn);
            (drop_fn)(null_mut());
            mem::forget(self);
            value
        }
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F) {
        f(self.mem);
        let drop_fn = ptr::read(&*self.drop_fn);
        (drop_fn)(null_mut());
        mem::forget(self);
    }
}

impl<'a, DropFn: FnOnce(*mut u8)> Drop for AnyValueTemp<'a, DropFn>{
    #[inline]
    fn drop(&mut self) {
        unsafe{
            let drop_fn = ptr::read(&*self.drop_fn);
            (drop_fn)(self.mem.as_ptr());
        }
    }
}