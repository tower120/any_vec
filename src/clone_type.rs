//!
//! Trait object based compile-time dispatch.
//!

use crate::traits::*;

#[derive(Copy, Clone, Default)]
pub struct Empty;

pub type CloneFn = fn(*const u8, *mut u8, usize);
fn clone_fn<T: Clone>(src: *const u8, dst: *mut u8, len: usize){
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
const fn get_clone_fn<T: Clone>() -> Option<CloneFn>{
    if impls::impls!(T: Copy){
        None
    } else {
        Some(clone_fn::<T>)
    }
}
pub trait CloneFnTrait<Traits: ?Sized>{
    const CLONE_FN: Option<CloneFn> = None;
}
impl<T: Clone> CloneFnTrait<dyn Cloneable> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}
impl<T: Clone> CloneFnTrait<dyn Cloneable+Send> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}
impl<T: Clone> CloneFnTrait<dyn Cloneable+Sync> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}
impl<T: Clone> CloneFnTrait<dyn Cloneable+Send+Sync> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}
impl<T> CloneFnTrait<dyn EmptyTrait> for T{}
impl<T> CloneFnTrait<dyn Send> for T{}
impl<T> CloneFnTrait<dyn Sync> for T{}
impl<T> CloneFnTrait<dyn Send+Sync> for T{}


/// This all just to replace AnyVec's clone function pointer with ZST,
/// when non-Cloneable.
pub trait CloneType{
    type Type: Copy;
    fn new(f: Option<CloneFn>) -> Self::Type;
    fn get(f: Self::Type) -> Option<CloneFn>;
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
impl_clone_type_empty!(dyn EmptyTrait);
impl_clone_type_empty!(dyn Sync);
impl_clone_type_empty!(dyn Send);
impl_clone_type_empty!(dyn Send + Sync);
impl_clone_type_fn!(dyn Cloneable);
impl_clone_type_fn!(dyn Cloneable + Send);
impl_clone_type_fn!(dyn Cloneable + Sync);
impl_clone_type_fn!(dyn Cloneable + Send + Sync);
