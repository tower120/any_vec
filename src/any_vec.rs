use std::alloc::Layout;
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use crate::{AnyVecMut, AnyVecRef};
use crate::any_value::AnyValue;
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{AnyValueTemp, Remove, SwapRemove};
use crate::any_vec::traits::{EmptyTrait};
use crate::clone_type::{clone_fn, CloneFn, CloneType};
use crate::traits::{Cloneable, Trait};

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
    /// Marker trait, for traits accepted by AnyVec.
    pub trait Trait: crate::clone_type::CloneType{}
    impl Trait for dyn EmptyTrait{}
    impl Trait for dyn Sync{}
    impl Trait for dyn Send{}
    impl Trait for dyn Sync + Send{}
    impl Trait for dyn Cloneable{}
    impl Trait for dyn Cloneable + Send{}
    impl Trait for dyn Cloneable + Sync{}
    impl Trait for dyn Cloneable + Send+ Sync{}

    /// Does not enforce anything. Default.
    pub trait EmptyTrait{}

    pub use std::marker::Sync;

    pub use std::marker::Send;

    /// Enforce type [`Clone`]-ability.
    pub trait Cloneable{}
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

impl<T> SatisfyTraits<dyn EmptyTrait> for T{}
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
pub struct AnyVec<Traits: ?Sized + Trait = dyn EmptyTrait>
{
    raw: AnyVecRaw,
    clone_fn: <Traits as CloneType>::Type,
    phantom: PhantomData<Traits>
}

// TODO: trait AnyVec with most functions ?

impl<Traits: ?Sized + Trait> AnyVec<Traits>
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

unsafe impl<Traits: ?Sized + Send + Trait> Send for AnyVec<Traits> {}

unsafe impl<Traits: ?Sized + Sync + Trait> Sync for AnyVec<Traits> {}

impl<Traits: ?Sized + Cloneable + Trait> Clone for AnyVec<Traits>
{
    fn clone(&self) -> Self {
        let clone_fn = <Traits as CloneType>::get(self.clone_fn);
        Self{
            raw: unsafe{ self.raw.clone(clone_fn) },
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }
}