mod wrapper;
mod raw;
mod lazy_clone;

pub use lazy_clone::{LazyClone};
pub use wrapper::AnyValueWrapper;
pub use raw::AnyValueRaw;

use std::any::TypeId;
use std::{mem, ptr, slice};
use std::mem::MaybeUninit;
use crate::copy_bytes_nonoverlapping;

/// Marker for unknown type.
pub struct Unknown;
impl Unknown {
    #[inline]
    pub fn is<T:'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<Unknown>()
    }
}

/// Type erased value interface.
pub trait AnyValue {
    /// Concrete type, or [`Unknown`]
    type Type: 'static /*= Unknown*/;

    fn value_typeid(&self) -> TypeId;

    fn as_bytes(&self) -> &[u8];

    #[inline]
    fn downcast_ref<T: 'static>(&self) -> Option<&T>{
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_ref_unchecked::<T>() })
        }
    }

    #[inline]
    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T{
        &*(self.as_bytes().as_ptr() as *const T)
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

    #[inline]
    unsafe fn downcast_unchecked<T: 'static>(self) -> T
        where Self: Sized
    {
        let mut tmp = MaybeUninit::<T>::uninit();
        self.move_into(tmp.as_mut_ptr() as *mut u8);
        tmp.assume_init()
    }

    /// Move self into `out`.
    ///
    /// `out` must have at least [`size`] bytes.
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

/// Helper function, which utilize type knowledge.
#[inline]
pub(crate) unsafe fn copy_bytes<T: AnyValue>(any_value: &T, out: *mut u8){
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

/// Type erased mutable value interface.
pub trait AnyValueMut: AnyValue{
    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8];
    // {
    //     let bytes = self.as_bytes();
    //     unsafe{slice::from_raw_parts_mut(
    //         bytes.as_ptr() as *mut u8,
    //         bytes.len()
    //     )}
    // }

    #[inline]
    fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T>{
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_mut_unchecked::<T>() })
        }
    }

    #[inline]
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T{
        &mut *(self.as_bytes_mut().as_mut_ptr() as *mut T)
    }
}

/// [`LazyClone`] friendly [`AnyValue`].
pub trait AnyValueCloneable: AnyValue {
    unsafe fn clone_into(&self, out: *mut u8);

    #[inline]
    fn lazy_clone(&self) -> LazyClone<Self>
        where Self: Sized
    {
        LazyClone::new(self)
    }
}
