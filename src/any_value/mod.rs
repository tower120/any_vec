mod wrapper;
mod raw;

pub use wrapper::AnyValueWrapper;
pub use raw::AnyValueRaw;

use std::any::TypeId;
use std::ptr;
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
///
/// Use [`consume_bytes`], if you need to read value.
/// Use [`consume_bytes_into`], if you need to copy value.
pub trait AnyValue {
    /// Concrete type, or [`Unknown`]
    type Type: 'static /*= Unknown*/;

    fn value_typeid(&self) -> TypeId;

    /// In bytes. Return compile-time value, whenever possible.
    fn value_size(&self) -> usize;

    // TODO: -> Option<T> , instead of panic
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
            self.consume_bytes_into(tmp.as_mut_ptr() as *mut u8);
            tmp.assume_init()
        }
    }

    // TODO: *const u8 ?
    /// Consume value as bytes.
    /// It is your responsibility to properly drop it.
    /// `size = size_of::<T>`
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F);


    /// `out` must have at least [`value_size`] bytes.
    /// Will do compile-time optimisation if type/size known.
    #[inline]
    unsafe fn consume_bytes_into(self, out: *mut u8)
        where Self: Sized
    {
        if !Unknown::is::<Self::Type>() {
            self.consume_bytes(|bytes| {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr() as *const Self::Type,
                    out as *mut Self::Type,
                    1);
            });
        } else {
            let size = self.value_size();
            self.consume_bytes(|bytes| {
                copy_bytes_nonoverlapping(bytes.as_ptr(), out, size);
            });

        }
    }
}