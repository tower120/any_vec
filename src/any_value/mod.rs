mod wrapper;
mod raw;
mod lazy_clone;

pub use lazy_clone::LazyClone;
pub use wrapper::AnyValueWrapper;
pub use raw::AnyValueRaw;

use std::any::TypeId;
use std::{mem, ptr};
use std::mem::MaybeUninit;
use std::ptr::NonNull;
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

    /// In bytes. Return compile-time value, whenever possible.
    fn size(&self) -> usize;

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

    // TODO: -> NonNull
    fn bytes(&self) -> *const u8;

    // TODO: bytes_mut, downcast_ref, downcast_mut

    /// `out` must have at least [`size`] bytes.
    /// Will do compile-time optimisation if type/size known.
    ///
    /// [`size`]: Self::size
    unsafe fn move_into(self, out: *mut u8)
        where Self: Sized
    {
        copy_bytes(&self, out);
        mem::forget(self);
    }
}

/// Helper function, which utilize type knowledge.
pub(crate) unsafe fn copy_bytes<T: AnyValue>(any_value: &T, out: *mut u8){
    if !Unknown::is::<T::Type>() {
        ptr::copy_nonoverlapping(
            any_value.bytes() as *const T::Type,
            out as *mut T::Type,
            1);
    } else {
        copy_bytes_nonoverlapping(
            any_value.bytes(),
            out,
            any_value.size());
    }
}

pub trait AnyValueCloneable: AnyValue {
    unsafe fn clone_into(&self, out: *mut u8);

    #[inline]
    fn lazy_clone(&self) -> LazyClone<Self> {
        LazyClone::new(self)
    }
}