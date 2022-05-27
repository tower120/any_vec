mod wrapper;
mod raw;

pub use wrapper::AnyValueWrapper;
pub use raw::AnyValueRaw;

use std::any::TypeId;
use std::ptr;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

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

    // TODO: *const u8
    /// Consume value as bytes.
    /// It is your responsibility to properly drop it.
    /// `size = size_of::<T>`
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F);
}