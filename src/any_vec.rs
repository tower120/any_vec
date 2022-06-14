use std::alloc::Layout;
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Range, RangeBounds};
use std::ptr::NonNull;
use std::slice;
use crate::{AnyVecTyped, into_range, refs};
use crate::any_value::{AnyValue};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{TempValue, SwapRemove, remove, Remove, swap_remove, Drain};
use crate::any_vec::traits::{None};
use crate::clone_type::{CloneFn, CloneFnTrait, CloneType};
use crate::element::{Element, ElementMut, ElementRef};
use crate::any_vec_ptr::AnyVecPtr;
use crate::iter::{Iter, IterMut, IterRef};
use crate::ops::splice::Splice;
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

/// Trait for compile time check - does `T` satisfy `Traits` constraints.
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
    #[inline]
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

    /// Same as clone, but without data copy.
    ///
    /// Since it does not copy underlying data, it works with any [`AnyVec`].
    /// Use it to construct [`AnyVec`] of the same type.
    #[inline]
    pub fn clone_empty(&self) -> Self {
        Self {
            raw: self.raw.clone_empty(),
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }

    #[inline]
    pub(crate) fn clone_fn(&self) -> Option<CloneFn>{
        <Traits as CloneType>::get(self.clone_fn)
    }

    #[inline]
    pub fn downcast_ref<Element: 'static>(&self) -> Option<AnyVecRef<Element>> {
        if self.element_typeid() == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<Element: 'static>(&self) -> AnyVecRef<Element> {
        refs::Ref(AnyVecTyped::new(NonNull::from(&self.raw)))
    }

    #[inline]
    pub fn downcast_mut<Element: 'static>(&mut self) -> Option<AnyVecMut<Element>> {
        if self.element_typeid() == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<Element: 'static>(&mut self) -> AnyVecMut<Element> {
        refs::Mut(AnyVecTyped::new(NonNull::from(&mut self.raw)))
    }

    #[inline]
    pub fn as_bytes(&self) -> *const u8 {
        self.raw.mem.as_ptr()
    }

    #[inline]
    pub fn iter(&self) -> IterRef<Traits>{
        Iter::new(AnyVecPtr::from(self), 0, self.len())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<Traits>{
        let len = self.len();
        Iter::new(AnyVecPtr::from(self), 0, len)
    }

    #[inline]
    unsafe fn get_element(&self, index: usize) -> ManuallyDrop<Element<AnyVecPtr<Traits>>>{
        let element = NonNull::new_unchecked(
            self.as_bytes().add(self.element_layout().size() * index) as *mut u8
        );
        ManuallyDrop::new(Element::new(
            AnyVecPtr::from(self),
            element
        ))
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<ElementRef<Traits>>{
        if index < self.len(){
            Some(unsafe{ self.get_unchecked(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> ElementRef<Traits>{
        refs::Ref(self.get_element(index))
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<ElementMut<Traits>>{
        if index < self.len(){
            Some(unsafe{ self.get_unchecked_mut(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> ElementMut<Traits> {
        refs::Mut(self.get_element(index))
    }

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if index is out of bounds.
    /// * Panics if out of memory.
    pub fn insert<V: AnyValue>(&mut self, index: usize, value: V) {
        self.raw.type_check(&value);
        unsafe{
            self.raw.insert_unchecked(index, value);
        }
    }

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if out of memory.
    #[inline]
    pub fn push<V: AnyValue>(&mut self, value: V) {
        self.raw.type_check(&value);
        unsafe{
            self.raw.push_unchecked(value);
        }
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

    /// Removes the specified range from the vector in bulk, returning all removed
    /// elements as an iterator. If the iterator is dropped before being fully consumed,
    /// it drops the remaining removed elements.
    ///
    /// The returned iterator keeps a mutable borrow on the vector.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if the end point
    /// is greater than the length of the vector.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements with indices in and past the range.
    ///
    /// [`mem::forget`]: std::mem::forget
    ///
    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Drain<Traits> {
        let Range{start, end} = into_range(self.len(), range);
        Drain::new(AnyVecPtr::from(self), start, end)
    }

    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    pub fn splice<I: IntoIterator>(&mut self, range: impl RangeBounds<usize>, replace_with: I)
        -> Splice<AnyVecPtr<Traits>, I::IntoIter>
    where
        I::IntoIter: ExactSizeIterator,
        I::Item: AnyValue
    {
        let Range{start, end} = into_range(self.len(), range);
        Splice::new(AnyVecPtr::from(self), start, end, replace_with.into_iter())
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

impl<'a, Traits: ?Sized + Trait> IntoIterator for &'a AnyVec<Traits>{
    type Item = ElementRef<'a, Traits>;
    type IntoIter = IterRef<'a, Traits>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Traits: ?Sized + Trait> IntoIterator for &'a mut AnyVec<Traits>{
    type Item = ElementMut<'a, Traits>;
    type IntoIter = IterMut<'a, Traits>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Typed view to &[`AnyVec`].
///
/// You can get it from [`AnyVec::downcast_ref`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::downcast_ref`]: crate::AnyVec::downcast_ref
pub type AnyVecRef<'a, T> = refs::Ref<AnyVecTyped<'a, T>>;

impl<'a, T: 'static> IntoIterator for AnyVecRef<'a, T>{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Typed view to &mut [`AnyVec`].
///
/// You can get it from [`AnyVec::downcast_mut`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::downcast_mut`]: crate::AnyVec::downcast_mut
pub type AnyVecMut<'a, T> = refs::Mut<AnyVecTyped<'a, T>>;

impl<'a, T: 'static> IntoIterator for AnyVecMut<'a, T>{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        self.iter_mut()
    }
}