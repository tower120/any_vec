//! AnyValue is concept of type-erased object, that
//! can be moved around without type knowledge.
//! 
//! With default trait implementation, all "consume" operations
//! boils down to [move_into]. By redefining [move_into] and [Drop][^1] behavior -
//! you can have some additional logic on AnyValue consumption.
//! (Consumption - is any operation that "move out" data from AnyValue)
//!
//! [AnyValueSizeless] -> [AnyValueTypeless] -> [AnyValue]
//!
//! # Usage
//! 
//! Some [AnyVec] operations will accept and return [AnyValue].
//! This allows to move data between [AnyVec]s in
//! fast, safe, type erased way.
//!
//! [^1]: AnyValue could have blanket implementation for Drop as well,
//!       but that is unstable Rust now.
//! 
//! [AnyVec]: crate::AnyVec

mod wrapper;
mod raw;
mod lazy_clone;

pub use lazy_clone::LazyClone;
pub use wrapper::AnyValueWrapper;
pub use raw::{AnyValueRaw, AnyValueTypelessRaw, AnyValueSizelessRaw};

use std::any::TypeId;
use std::{mem, ptr};
use std::mem::{MaybeUninit, size_of};
use crate::{copy_bytes_nonoverlapping, swap_bytes_nonoverlapping};

/// Marker for unknown type.
pub struct Unknown;
impl Unknown {
    #[inline]
    pub fn is<T:'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<Unknown>()
    }
}

/// Prelude for traits.
pub mod traits{
    pub use super::{AnyValueSizeless, AnyValueSizelessMut};
    pub use super::{AnyValueTypeless, AnyValueTypelessMut};
    pub use super::{AnyValue, AnyValueMut};
    pub use super::AnyValueCloneable;
}

/// [AnyValue] that doesn't know its size or type, and can provide only object ptr.
pub trait AnyValueSizeless {
    /// Concrete type, or [`Unknown`]
    ///
    /// N.B. This should be in `AnyValueTyped`. It is here due to ergonomic reasons,
    /// since Rust does not have impl specialization.
    type Type: 'static /*= Unknown*/;

    /// Aligned address.
    fn as_bytes_ptr(&self) -> *const u8;

    #[inline]
    unsafe fn downcast_ref_unchecked<T>(&self) -> &T{
        &*(self.as_bytes_ptr() as *const T)
    }

    #[inline]
    unsafe fn downcast_unchecked<T: 'static>(self) -> T
        where Self: Sized
    {
        let mut tmp = MaybeUninit::<T>::uninit();
        self.move_into::<T>(tmp.as_mut_ptr() as *mut u8, size_of::<T>());
        tmp.assume_init()
    }

    /// Move self into `out`.
    ///
    /// Will do compile-time optimisation if type/size known.
    ///
    /// # Safety
    ///
    /// `bytes_size` must be correct object size.
    /// `out` must have at least `bytes_size` bytes.
    /// `KnownType` must be correct object type or [Unknown].
    ///
    /// Use [move_into] as safe version.
    #[inline]
    unsafe fn move_into<KnownType:'static /*= Unknown*/>(self, out: *mut u8, bytes_size: usize)
        where Self: Sized
    {
        copy_bytes::<KnownType>(self.as_bytes_ptr(), out, bytes_size);
        mem::forget(self);
    }
}

/// Move AnyValue into `out` location.
///
/// If `T` has known [Type] compile time optimizations will be applied.
///
/// [Type]: AnyValueSizeless::Type
#[inline]
pub fn move_into<T: AnyValueTypeless>(this: T, out: *mut u8) {
    let size = this.as_bytes().len();
    unsafe{
        move_into_w_size::<T>(this, out, size);
    }
}

/// [move_into] but with `bytes_size` hint.
///
/// In loops, compiler may generate more optimized code, if will
/// know that the same size is used for all moves.
/// Acts the same as [move_into] if [Type] is known.
///
/// [Type]: AnyValueSizeless::Type
#[inline]
pub unsafe fn move_into_w_size<T: AnyValueSizeless>(this: T, out: *mut u8, bytes_size: usize) {
    copy_bytes::<T::Type>(this.as_bytes_ptr(), out, bytes_size);
    mem::forget(this);
}

/// [AnyValue] that doesn't know it's type, but know it's size.
pub trait AnyValueTypeless: AnyValueSizeless {
    /// Aligned.
    #[inline]
    fn as_bytes(&self) -> &[u8]{
        unsafe{std::slice::from_raw_parts(
            self.as_bytes_ptr(),
            self.size()
        )}
    }

    /// Aligned.
    fn size(&self) -> usize;
}

/// Type erased value interface.
/// 
/// Know it's type and size, possibly compile-time.
pub trait AnyValue: AnyValueTypeless {
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
pub(crate) unsafe fn copy_bytes<KnownType: 'static>(
    input: *const u8, out: *mut u8, bytes_size: usize
) {
    if !Unknown::is::<KnownType>() {
        ptr::copy_nonoverlapping(
            input as *const KnownType,
            out as *mut KnownType,
            1);
    } else {
        copy_bytes_nonoverlapping(
            input,
            out,
            bytes_size);
    }
}

/// Mutable [AnyValueSizeless].
pub trait AnyValueSizelessMut: AnyValueSizeless {
    // Rust MIRI requires mut pointer to actually come from mut self.
    /// Aligned address.
    fn as_bytes_mut_ptr(&mut self) -> *mut u8;

    #[inline]
    unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut T{
        &mut *(self.as_bytes_mut_ptr() as *mut T)
    }
}

/// Mutable [AnyValueTypeless].
pub trait AnyValueTypelessMut: AnyValueTypeless + AnyValueSizelessMut {
    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8]{
        unsafe{std::slice::from_raw_parts_mut(
            self.as_bytes_mut_ptr(),
            self.size()
        )}
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

/// Mutable [AnyValue].
pub trait AnyValueMut: AnyValueTypelessMut + AnyValue {
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

/// [`LazyClone`] friendly [`AnyValueSizeless`].
pub trait AnyValueCloneable: AnyValueSizeless {
    unsafe fn clone_into(&self, out: *mut u8);

    #[inline]
    fn lazy_clone(&self) -> LazyClone<Self>
        where Self: Sized
    {
        LazyClone::new(self)
    }
}
