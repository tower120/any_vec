//! AnyValue is concept of type-erased object, that
//! can be moved around without type knowledge.
//!
//! [AnyValuePtr] -> [AnyValueSized] -> [AnyValueTyped]
//!
//! Some [AnyVec] operations will return [AnyValueTyped],
//! [AnyVec] operations also accept AnyValue as arguments.
//! This allows to move data between [AnyVec]s in
//! fast, safe, type erased way.
//!
//! [AnyVec]: crate::AnyVec

mod wrapper;
mod raw;
mod lazy_clone;

pub use lazy_clone::{LazyClone};
pub use wrapper::AnyValueWrapper;
pub use raw::{AnyValueRawTyped, AnyValueRawSized, AnyValueRawPtr};

use std::any::TypeId;
use std::{mem, ptr};
use std::mem::{MaybeUninit, size_of};
use std::ptr::NonNull;
use crate::{copy_bytes_nonoverlapping, swap_bytes_nonoverlapping};

/// Marker for unknown type.
pub struct Unknown;
impl Unknown {
    #[inline]
    pub fn is<T:'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<Unknown>()
    }
}

/// AnyValue that can provide only object ptr.
pub trait AnyValuePtr {
    /// Concrete type, or [`Unknown`]
    ///
    /// N.B. This should be in `AnyValueTyped`. It is here due to ergonomic reasons,
    /// since Rust does not have impl specialization.
    type Type: 'static /*= Unknown*/;

    /// Aligned address.
    fn as_bytes_ptr(&self) -> NonNull<u8>;

    #[inline]
    unsafe fn downcast_ref_unchecked<T>(&self) -> &T{
        &*(self.as_bytes_ptr().as_ptr() as *const T)
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
/// [Type]: AnyValuePtr::Type
#[inline]
pub fn move_into<T: AnyValueSized>(this: T, out: *mut u8) {
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
/// [Type]: AnyValuePtr::Type
#[inline]
pub unsafe fn move_into_w_size<T: AnyValuePtr>(this: T, out: *mut u8, bytes_size: usize) {
    copy_bytes::<T::Type>(this.as_bytes_ptr(), out, bytes_size);
    mem::forget(this);
}

/// [AnyValuePtr] that know it's size.
pub trait AnyValueSized: AnyValuePtr {
    /// Aligned.
    #[inline]
    fn as_bytes(&self) -> &[u8]{
        unsafe{std::slice::from_raw_parts(
            self.as_bytes_ptr().as_ptr(),
            self.size()
        )}
    }

    /// Aligned.
    fn size(&self) -> usize;
}

/// [AnyValueSized] that know it's type, possibly compiletime.
pub trait AnyValueTyped: AnyValueSized {
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
    input: NonNull<u8>, out: *mut u8, bytes_size: usize
) {
    if !Unknown::is::<KnownType>() {
        ptr::copy_nonoverlapping(
            input.as_ptr() as *const KnownType,
            out as *mut KnownType,
            1);
    } else {
        copy_bytes_nonoverlapping(
            input.as_ptr(),
            out,
            bytes_size);
    }
}

/// Mutable [AnyValuePtr].
pub trait AnyValuePtrMut: AnyValuePtr {
    #[inline]
    unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut T{
        &mut *(self.as_bytes_ptr().as_ptr() as *mut T)
    }
}

/// Mutable [AnyValueSized].
pub trait AnyValueSizedMut: AnyValueSized + AnyValuePtrMut {
    #[inline]
    fn as_bytes_mut(&mut self) -> &mut [u8]{
        unsafe{std::slice::from_raw_parts_mut(
            self.as_bytes_ptr().as_ptr(),
            self.size()
        )}
    }

    #[inline]
    unsafe fn swap_unchecked<Other: AnyValueTypedMut>(&mut self, other: &mut Other){
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

/// Mutable [AnyValueTyped].
pub trait AnyValueTypedMut: AnyValueSizedMut + AnyValueTyped {
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
    fn swap<Other: AnyValueTypedMut>(&mut self, other: &mut Other){
        assert_eq!(self.value_typeid(), other.value_typeid());
        unsafe{
            self.swap_unchecked(other);
        }
    }
}

/// [`LazyClone`] friendly [`AnyValuePtr`].
pub trait AnyValueCloneable: AnyValuePtr {
    unsafe fn clone_into(&self, out: *mut u8);

    #[inline]
    fn lazy_clone(&self) -> LazyClone<Self>
        where Self: Sized
    {
        LazyClone::new(self)
    }
}
