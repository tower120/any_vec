use std::any::{Any, TypeId};
use std::{mem, ptr};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit, size_of};
use std::ops::DerefMut;
use std::ptr::{drop_in_place, NonNull, null_mut};

pub(crate) mod temp;
mod wrapper;

pub use temp::AnyValueTemp;
pub use wrapper::AnyValueWrapper;

pub trait AnyValue {
    /// Concrete type, or [`Unknown`]
    ///
    /// [`Unknown`]: crate::Unknown
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