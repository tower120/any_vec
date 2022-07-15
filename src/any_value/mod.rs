mod wrapper;
mod raw;
mod lazy_clone;

pub use lazy_clone::{LazyClone};
pub use wrapper::AnyValueWrapper;
pub use raw::{AnyValueRaw, AnyValueRawUnknown};

use std::any::TypeId;
use std::{mem, ptr};
use std::mem::MaybeUninit;
use crate::{copy_bytes_nonoverlapping, swap_bytes_nonoverlapping};

/// Marker for unknown type.
pub struct Unknown;
impl Unknown {
    #[inline]
    pub fn is<T:'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<Unknown>()
    }
}

/// [`AnyValue`] that does not know it's compiletime or runtime type.
pub trait AnyValueUnknown {
    /// Concrete type, or [`Unknown`]
    ///
    /// N.B. This should be in `AnyValue`. It is here due to ergonomic reasons,
    /// since Rust does not have impl specialization.
    type Type: 'static /*= Unknown*/;

    /// Aligned.
    fn as_bytes(&self) -> &[u8];

    #[inline]
    unsafe fn downcast_ref_unchecked<T>(&self) -> &T{
        &*(self.as_bytes().as_ptr() as *const T)
    }

    #[inline]
    unsafe fn downcast_unchecked<T>(self) -> T
        where Self: Sized
    {
        let mut tmp = MaybeUninit::<T>::uninit();
        self.move_into(tmp.as_mut_ptr() as *mut u8);
        tmp.assume_init()
    }

    /// Move self into `out`.
    ///
    /// `out` must have at least `as_bytes().len()` bytes.
    /// Will do compile-time optimisation if type/size known.
    ///
    /// [`size`]: Self::size
    #[inline]
    unsafe fn move_into(self, out: *mut u8)
        where Self: Sized
    {
        copy_bytes(&self, out);
        mem::forget(self);
    }
}

/// Type erased value interface.
pub trait AnyValue: AnyValueUnknown {
    fn value_typeid(&self) -> TypeId;

    #[inline]
    fn downcast_ref<T: 'static>(&self) -> Option<&T>{
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_ref_unchecked::<T>() })
        }
    }

    #[inline]
    fn downcast<T: 'static>(self) -> Option<T>
        where Self: Sized
    {
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_unchecked::<T>() })
        }
    }
}

/// Helper function, which utilize type knowledge.
#[inline]
pub(crate) unsafe fn copy_bytes<T: AnyValueUnknown>(any_value: &T, out: *mut u8){
    if !Unknown::is::<T::Type>() {
        ptr::copy_nonoverlapping(
            any_value.as_bytes().as_ptr() as *const T::Type,
            out as *mut T::Type,
            1);
    } else {
        let bytes = any_value.as_bytes();
        copy_bytes_nonoverlapping(
            bytes.as_ptr(),
            out,
            bytes.len());
    }
}

/// [`AnyValueMut`] that does not know it's compiletime or runtime type.
pub trait AnyValueMutUnknown: AnyValueUnknown {
    fn as_bytes_mut(&mut self) -> &mut [u8];

    #[inline]
    unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut T{
        &mut *(self.as_bytes_mut().as_mut_ptr() as *mut T)
    }

    #[inline]
    unsafe fn swap_unchecked<Other: AnyValueMut>(&mut self, other: &mut Other){
        // compile-time check
        if !Unknown::is::<Self::Type>() {
            mem::swap(
                self.downcast_mut_unchecked::<Self::Type>(),
                other.downcast_mut_unchecked::<Self::Type>()
            );
        } else if !Unknown::is::<Other::Type>() {
            mem::swap(
                self.downcast_mut_unchecked::<Other::Type>(),
                other.downcast_mut_unchecked::<Other::Type>()
            );
        } else {
            let bytes = self.as_bytes_mut();
            swap_bytes_nonoverlapping(
                bytes.as_mut_ptr(),
                other.as_bytes_mut().as_mut_ptr(),
                bytes.len()
            );
        }
    }
}

/// Type erased mutable value interface.
pub trait AnyValueMut: AnyValueMutUnknown + AnyValue{
    #[inline]
    fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T>{
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_mut_unchecked::<T>() })
        }
    }

    /// Swaps underlying values.
    ///
    /// # Panic
    ///
    /// Panics, if type mismatch.
    #[inline]
    fn swap<Other: AnyValueMut>(&mut self, other: &mut Other){
        assert_eq!(self.value_typeid(), other.value_typeid());
        unsafe{
            self.swap_unchecked(other);
        }
    }
}

/// [`LazyClone`] friendly [`AnyValueUnknown`].
pub trait AnyValueCloneable: AnyValueUnknown {
    unsafe fn clone_into(&self, out: *mut u8);

    #[inline]
    fn lazy_clone(&self) -> LazyClone<Self>
        where Self: Sized
    {
        LazyClone::new(self)
    }
}
