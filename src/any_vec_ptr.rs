//! Type dispatched analog of `enum{*AnyVecRaw, *AnyVec<Traits>}`.

use std::ptr::NonNull;
use crate::any_vec_raw::AnyVecRaw;
use crate::AnyVec;
use crate::traits::Trait;

pub trait IAnyVecRawPtr: Copy{
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw>;
}
pub trait IAnyVecPtr<Traits: ?Sized + Trait>: IAnyVecRawPtr{
    fn any_vec(&self) -> NonNull<AnyVec<Traits>>;
}

#[derive(Copy, Clone)]
pub struct AnyVecRawPtr{
    ptr: NonNull<AnyVecRaw>
}
impl From<NonNull<AnyVecRaw>> for AnyVecRawPtr {
    #[inline]
    fn from(ptr: NonNull<AnyVecRaw>) -> Self {
        Self{ptr}
    }
}
impl IAnyVecRawPtr for AnyVecRawPtr{
    #[inline]
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw> {
        self.ptr
    }
}


pub struct AnyVecPtr<Traits: ?Sized + Trait>{
    ptr: NonNull<AnyVec<Traits>>
}
impl<Traits: ?Sized + Trait> From<NonNull<AnyVec<Traits>>> for AnyVecPtr<Traits> {
    #[inline]
    fn from(ptr: NonNull<AnyVec<Traits>>) -> Self {
        Self{ptr}
    }
}
impl<Traits: ?Sized + Trait> From<&mut AnyVec<Traits>> for AnyVecPtr<Traits> {
    #[inline]
    fn from(reference: &mut AnyVec<Traits>) -> Self {
        Self{ptr: NonNull::from(reference)}
    }
}
impl<Traits: ?Sized + Trait> From<&AnyVec<Traits>> for AnyVecPtr<Traits> {
    #[inline]
    fn from(reference: &AnyVec<Traits>) -> Self {
        Self{ptr: NonNull::from(reference)}
    }
}
impl<Traits: ?Sized + Trait> Clone for AnyVecPtr<Traits>{
    #[inline]
    fn clone(&self) -> Self {
        Self{ptr: self.ptr}
    }
}
impl<Traits: ?Sized + Trait> Copy for AnyVecPtr<Traits>{}

impl<Traits: ?Sized + Trait> IAnyVecRawPtr for AnyVecPtr<Traits> {
    #[inline]
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw> {
        NonNull::from(unsafe{&self.ptr.as_ref().raw})
    }
}
impl<Traits: ?Sized + Trait> IAnyVecPtr<Traits> for AnyVecPtr<Traits> {
    #[inline]
    fn any_vec(&self) -> NonNull<AnyVec<Traits>> {
        self.ptr
    }
}
