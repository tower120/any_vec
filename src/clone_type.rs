//! This all just to replace AnyVec's clone function pointer with ZST,
//! when non-Cloneable.

use crate::traits::*;

pub type CloneFn = fn(*const u8, *mut u8, usize);
pub fn clone_fn<T: Clone>(src: *const u8, dst: *mut u8, len: usize){
    let src = src as *const T;
    let dst = dst as *mut T;
    for i in 0..len {
        unsafe{
            let dst = dst.add(i);
            let src = src.add(i);
            dst.write((*src).clone());
        }
    }
}

macro_rules! impl_clone_type_empty {
    ($t:ty) => {
        impl CloneType for $t {
            type Type = Empty;
            fn new(_: Option<CloneFn>) -> Self::Type{ Empty }
            fn get(f: Self::Type) -> Option<CloneFn>{ None }
        }
    }
}

macro_rules! impl_clone_type_fn {
    ($t:ty) => {
        impl CloneType for $t {
            type Type = Option<CloneFn>;
            fn new(f: Option<CloneFn>) -> Self::Type{ f }
            fn get(f: Self::Type) -> Option<CloneFn>{ f as Option<CloneFn> }
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Empty;

pub trait CloneType{
    type Type: Copy;
    fn new(f: Option<CloneFn>) -> Self::Type;
    fn get(f: Self::Type) -> Option<CloneFn>;
}
impl_clone_type_empty!(dyn EmptyTrait);
impl_clone_type_empty!(dyn Sync);
impl_clone_type_empty!(dyn Send);
impl_clone_type_empty!(dyn Send + Sync);
impl_clone_type_fn!(dyn Cloneable);
impl_clone_type_fn!(dyn Cloneable + Send);
impl_clone_type_fn!(dyn Cloneable + Sync);
impl_clone_type_fn!(dyn Cloneable + Send + Sync);
