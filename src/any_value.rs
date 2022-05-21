use std::any::{Any, TypeId};
use std::{mem, ptr};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit, size_of};
use std::ops::DerefMut;
use std::ptr::{drop_in_place, NonNull, null_mut};

pub trait AnyValue {
    // TODO: remove?
    /// Known type size.
    /// Used for optimization.
    //const KNOWN_SIZE: Option<usize> = None;

    /// Concrete type, or [`Unknown`]
    type Type: 'static /*= Unknown*/;

    fn value_typeid(&self) -> TypeId;

    /// # Panic
    ///
    /// Panics if type mismatch
    #[inline]
    fn downcast<T: 'static>(self) -> T
        where Self: Sized
    {
        assert_eq!(self.value_typeid(), TypeId::of::<T>());
        unsafe{
            let mut tmp = MaybeUninit::<T>::uninit();
            self.consume_bytes(|element|{
                ptr::copy_nonoverlapping(element.as_ptr() as *const T, tmp.as_mut_ptr(), 1);
            });
            tmp.assume_init()
        }
    }

    /// Consume value as bytes.
    /// It is your responsibility to properly drop it.
    /// `size = size_of::<T>`
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F);
}

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
    //const KNOWN_SIZE: Option<usize> = Some(size_of::<T>());
    type Type = T;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline]
    fn downcast<U: 'static>(self) -> U {
        assert_eq!(self.value_typeid(), TypeId::of::<U>());
        // rust don't see that types are the same after assert.
        unsafe {
            let ptr = &self.value as *const T as *const U;
            mem::forget(self.value);
            ptr::read(ptr)
        }
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(mut self, f: F) {
        f(NonNull::new_unchecked(&mut self.value as *mut _  as *mut u8));
        mem::forget(self.value);
    }
}