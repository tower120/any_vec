use std::alloc::Layout;
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use crate::{AnyVecMut, AnyVecRef};
use crate::any_value::AnyValue;
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{AnyValueTemp, Remove, SwapRemove};
use crate::any_vec::traits::{Trait};
use crate::traits::Cloneable;

// TODO: rename mod to marker
/// Trait constraints.
/// Possible variants [`Cloneable`], [`Send`] and [`Sync`], in any combination.
///
/// # Example
/// ```rust
/// use any_vec::AnyVec;
/// use any_vec::traits::*;
/// let v1: AnyVec<dyn Cloneable + Sync + Send> = AnyVec::new::<String>();
/// let v2 = v1.clone();
///
/// ```
pub mod traits{
    mod private{
        pub trait Sealed{}
    }

    /// Does not enforce anything. Default.
    pub trait Trait: private::Sealed {}

    impl Trait for dyn Sync{}
    impl private::Sealed for dyn Sync{}

    impl Trait for dyn Send{}
    impl private::Sealed for dyn Send{}

    impl Trait for dyn Sync + Send{}
    impl private::Sealed for dyn Sync + Send{}

    /// Enforce type [`Clone`]-ability.
    //pub trait Cloneable: Trait{}

    pub trait Cloneable{}
    impl Trait for dyn Cloneable{}
    impl private::Sealed for dyn Cloneable{}

    impl Trait for dyn Cloneable + Send{}
    impl private::Sealed for dyn Cloneable + Send{}

    impl Trait for dyn Cloneable + Sync{}
    impl private::Sealed for dyn Cloneable + Sync{}
    
    impl Trait for dyn Cloneable + Send + Sync{}
    impl private::Sealed for dyn Cloneable + Send + Sync{}
}


const fn get_clone_fn<T: Clone>() -> Option<CloneFn>{
    if impls::impls!(T: Copy){
        None
    } else {
        Some(clone_fn::<T>)
    }
}

pub trait SatisfyTraits<Traits: ?Sized>{
    const CLONE_FN: Option<CloneFn> = None;
}

impl<T> SatisfyTraits<dyn Trait> for T{}
impl<T: Clone> SatisfyTraits<dyn Cloneable> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}
impl<T: Send> SatisfyTraits<dyn Send> for T{}
impl<T: Sync> SatisfyTraits<dyn Sync> for T{}

impl<T: Send + Sync> SatisfyTraits<dyn Send + Sync> for T{}
impl<T: Clone + Send> SatisfyTraits<dyn Cloneable + Send> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}
impl<T: Clone + Sync> SatisfyTraits<dyn Cloneable + Sync> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}

impl<T: Clone + Send + Sync> SatisfyTraits<dyn Cloneable + Send + Sync> for T{
    const CLONE_FN: Option<CloneFn> = get_clone_fn::<T>();
}


pub(crate) type CloneFn = fn(*const u8, *mut u8, usize);
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
/*trait GetCloneFn<Traits: ?Sized>{
    const CLONE_FN: Option<CloneFn> = None;
}
impl<T: Clone> GetCloneFn<dyn Cloneable> for T {
    const CLONE_FN: Option<CloneFn> =
        if impls!(T: Copy){
            None
        } else {
            Some(clone_fn::<T>)
        };
}*/


macro_rules! impl_clone_type_empty {
    ($t:ty) => {
        impl CloneType for $t {
            type Type = Empty;
            fn new(_: Option<CloneFn>) -> Self::Type{ Empty }
            fn get_fn(f: Self::Type) -> Option<CloneFn>{ None }
        }
    }
}

macro_rules! impl_clone_type_fn {
    ($t:ty) => {
        impl CloneType for $t {
            type Type = Option<CloneFn>;
            fn new(f: Option<CloneFn>) -> Self::Type{ f }
            fn get_fn(f: Self::Type) -> Option<CloneFn>{ f as Option<CloneFn> }
        }
    }
}


#[derive(Copy, Clone, Default)]
pub struct Empty;

pub trait CloneType{
    type Type: Copy;
    fn new(f: Option<CloneFn>) -> Self::Type;
    fn get_fn(f: Self::Type) -> Option<CloneFn>;
}
impl_clone_type_empty!(dyn Trait);
impl_clone_type_empty!(dyn Sync);
impl_clone_type_empty!(dyn Send);
impl_clone_type_empty!(dyn Send + Sync);
impl_clone_type_fn!(dyn Cloneable);
impl_clone_type_fn!(dyn Cloneable + Send);
impl_clone_type_fn!(dyn Cloneable + Sync);
impl_clone_type_fn!(dyn Cloneable + Send + Sync);



pub trait TraitT: Trait + CloneType{}
impl TraitT for dyn Trait{}
impl TraitT for dyn Send{}
impl TraitT for dyn Sync{}
impl TraitT for dyn Send + Sync{}
impl TraitT for dyn Cloneable{}
impl TraitT for dyn Cloneable + Send{}
impl TraitT for dyn Cloneable + Sync{}
impl TraitT for dyn Cloneable + Send + Sync{}



/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// You can make AnyVec [`Send`]-able, [`Sync`]-able, [`Cloneable`], by
/// specifying trait constraints: `AnyVec<dyn Cloneable + Sync + Send>`. See [`crate::traits`].
///
/// Some operations return [`AnyValueTemp<Operation>`], which internally holds &mut to [`AnyVec`].
/// You can drop it, cast to concrete type, or put into another vector. (See [`AnyValue`])
///
/// *`Element: 'static` due to TypeId requirements*
pub struct AnyVec<Traits: ?Sized + /*Trait + CloneType*/TraitT = dyn Trait>
{
    raw: AnyVecRaw,
    clone_fn: <Traits as CloneType>::Type,
    phantom: PhantomData<Traits>
}

// TODO: trait AnyVec with most functions ?

impl<Traits: ?Sized + TraitT> AnyVec<Traits>
    where Traits: CloneType
{
    /// Element should implement requested Traits
    pub fn new<Element: 'static>() -> Self
        where Element: SatisfyTraits<Traits>
    {
        Self::with_capacity::<Element>(0)
    }

    /// Element should implement requested Traits
    pub fn with_capacity<Element: 'static>(capacity: usize) -> Self
        where Element: SatisfyTraits<Traits>
    {
        let clone_fn = <Element as SatisfyTraits<Traits>>::CLONE_FN;
        Self{
            raw: AnyVecRaw::with_capacity::<Element>(capacity),
            clone_fn: <Traits as CloneType>::new(clone_fn),
            phantom: PhantomData
        }
    }

    #[inline]
    pub fn downcast_ref<Element: 'static>(&self) -> Option<AnyVecRef<Element>> {
        self.raw.downcast_ref::<Element>()
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<Element: 'static>(&self) -> AnyVecRef<Element> {
        self.raw.downcast_ref_unchecked::<Element>()
    }

    #[inline]
    pub fn downcast_mut<Element: 'static>(&mut self) -> Option<AnyVecMut<Element>> {
        self.raw.downcast_mut::<Element>()
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<Element: 'static>(&mut self) -> AnyVecMut<Element> {
        self.raw.downcast_mut_unchecked::<Element>()
    }

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if index is out of bounds.
    /// * Panics if out of memory.
    pub fn insert<V: AnyValue>(&mut self, index: usize, value: V) {
        self.raw.insert(index, value);
    }

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if out of memory.
    #[inline]
    pub fn push<V: AnyValue>(&mut self, value: V) {
        self.raw.push(value);
    }

    /// # Panics
    ///
    /// * Panics if index out of bounds.
    ///
    /// # Leaking
    ///
    /// If the returned [`AnyValueTemp`] goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements with indices >= index.
    ///
    #[inline]
    pub fn remove(&mut self, index: usize) -> AnyValueTemp<Remove> {
        self.raw.remove(index)
    }

    /// # Panics
    ///
    /// * Panics if index out of bounds.
    ///
    /// # Leaking
    ///
    /// If the returned [`AnyValueTemp`] goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements with indices >= index.
    ///
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> AnyValueTemp<SwapRemove> {
        self.raw.swap_remove(index)
    }

    #[inline]
    pub fn clear(&mut self){
        self.raw.clear()
    }

    /// Element TypeId
    #[inline]
    pub fn element_typeid(&self) -> TypeId{
        self.raw.element_typeid()
    }

    /// Element Layout
    #[inline]
    pub fn element_layout(&self) -> Layout {
        self.raw.element_layout()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }
}

unsafe impl<Traits: ?Sized + TraitT> Send for AnyVec<Traits>
    where Traits: Send + CloneType
{}

unsafe impl<Traits: ?Sized + TraitT> Sync for AnyVec<Traits>
    where Traits: Sync + CloneType
{}

impl<Traits: ?Sized + TraitT> Clone for AnyVec<Traits>
    where Traits: Cloneable + CloneType
{
    fn clone(&self) -> Self {
        let clone_fn = <Traits as CloneType>::get_fn(self.clone_fn);
        Self{
            raw: unsafe{ self.raw.clone(clone_fn) },
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }
}