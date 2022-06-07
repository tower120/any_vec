use std::alloc::Layout;
use std::any::TypeId;
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::{AnyVecMut, AnyVecRef/*, LazyClonedElement, ElementRef*/};
use crate::any_value::{AnyValue};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{TempValue, SwapRemove, remove, Remove, swap_remove};
use crate::any_vec::traits::{None};
use crate::clone_type::{CloneFn, CloneFnTrait, CloneType};
use crate::element2::{Element, ElementMut, ElementRef};
use crate::ops::any_vec_ptr::AnyVecPtr;
use crate::traits::{Cloneable, Trait};

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
    impl Trait for dyn None {}
    impl Trait for dyn Sync{}
    impl Trait for dyn Send{}
    impl Trait for dyn Sync + Send{}
    impl Trait for dyn Cloneable{}
    impl Trait for dyn Cloneable + Send{}
    impl Trait for dyn Cloneable + Sync{}
    impl Trait for dyn Cloneable + Send+ Sync{}

    /// Does not enforce anything. Default.
    pub trait None {}

    pub use std::marker::Sync;

    pub use std::marker::Send;

    /// Enforce type [`Clone`]-ability.
    pub trait Cloneable{}
}

/// Trait for compile time check if T satisfy Traits constraints.
///
/// Almost for sure you don't need to use it. It is public - just in case.
/// In our tests we found niche case where it was needed:
/// ```rust
///     # use any_vec::AnyVec;
///     # use any_vec::SatisfyTraits;
///     # use any_vec::traits::*;
///     fn do_test<Traits: ?Sized + Cloneable + Trait>(vec: &mut AnyVec<Traits>)
///         where String: SatisfyTraits<Traits>,
///               usize:  SatisfyTraits<Traits>
///     {
///         # let something = true;
///         # let other_something = true;
///         if something {
///             *vec = AnyVec::new::<String>();
///             /*...*/
///         } else if other_something {
///             *vec = AnyVec::new::<usize>();
///             /*...*/
///         }
///     # }
/// ```
pub trait SatisfyTraits<Traits: ?Sized>: CloneFnTrait<Traits> {}
impl<T> SatisfyTraits<dyn None> for T{}
impl<T: Clone> SatisfyTraits<dyn Cloneable> for T{}
impl<T: Send> SatisfyTraits<dyn Send> for T{}
impl<T: Sync> SatisfyTraits<dyn Sync> for T{}
impl<T: Send + Sync> SatisfyTraits<dyn Send + Sync> for T{}
impl<T: Clone + Send> SatisfyTraits<dyn Cloneable + Send> for T{}
impl<T: Clone + Sync> SatisfyTraits<dyn Cloneable + Sync> for T{}
impl<T: Clone + Send + Sync> SatisfyTraits<dyn Cloneable + Send + Sync> for T{}


/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// You can make AnyVec [`Send`]-able, [`Sync`]-able, [`Cloneable`], by
/// specifying trait constraints: `AnyVec<dyn Cloneable + Sync + Send>`. See [`traits`].
///
/// Some operations return [`TempValue<Operation, Traits>`], which internally holds &mut to [`AnyVec`].
/// You can drop it, cast to concrete type, or put into another vector. (See [`AnyValue`])
///
/// *`Element: 'static` due to TypeId requirements*
pub struct AnyVec<Traits: ?Sized + Trait = dyn None>
{
    pub(crate) raw: AnyVecRaw,
    clone_fn: <Traits as CloneType>::Type,
    phantom: PhantomData<Traits>
}

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
        let clone_fn = <Element as CloneFnTrait<Traits>>::CLONE_FN;
        Self{
            raw: AnyVecRaw::with_capacity::<Element>(capacity),
            clone_fn: <Traits as CloneType>::new(clone_fn),
            phantom: PhantomData
        }
    }

    #[inline]
    pub(crate) fn clone_fn(&self) -> Option<CloneFn>{
        <Traits as CloneType>::get(self.clone_fn)
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

    #[inline]
    pub fn as_bytes(&self) -> *const u8 {
        self.raw.mem.as_ptr()
    }

    #[inline]
    pub fn get(&self, index: usize) -> ElementRef<Traits>{
        self.raw.index_check(index);
        unsafe{
            self.get_unchecked(index)
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> ElementRef<Traits>{
        ElementRef(
            Element{
                any_vec: self,
                element: self.as_bytes().add(self.element_layout().size() * index)
            }
        )
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> ElementMut<Traits>{
        self.raw.index_check(index);
        unsafe{
            self.get_mut_unchecked(index)
        }
    }

    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self, index: usize) -> ElementMut<Traits>{
         ElementMut(
            Element{
                any_vec: self,
                element: self.as_bytes().add(self.element_layout().size() * index)
            }
        )
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
    /// If the returned [`TempValue`] goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements with indices >= index.
    ///
    /// [`mem::forget`]: std::mem::forget
    ///
    #[inline]
    pub fn remove(&mut self, index: usize) -> Remove<Traits> {
        self.raw.index_check(index);
        TempValue::new(remove::Remove::new(
            AnyVecPtr::from(self),
            index
        ))
    }

    /// # Panics
    ///
    /// * Panics if index out of bounds.
    ///
    /// # Leaking
    ///
    /// If the returned [`TempValue`] goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements with indices >= index.
    ///
    /// [`mem::forget`]: std::mem::forget
    ///
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> SwapRemove<Traits> {
        self.raw.index_check(index);
        TempValue::new(swap_remove::SwapRemove::new(
            AnyVecPtr::from(self),
            index
        ))
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
        self.raw.len
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
        Self{
            raw: unsafe{ self.raw.clone(self.clone_fn()) },
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }
}