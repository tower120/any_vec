use std::alloc::Layout;
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut, Range, RangeBounds};
use std::ptr::NonNull;
use std::slice;
use crate::{AnyVecTyped, into_range, mem, ops};
use crate::any_value::{AnyValue};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{TempValue, Remove, SwapRemove, remove, swap_remove};
use crate::ops::{Drain, Splice, drain, splice};
use crate::any_vec::traits::{None};
use crate::clone_type::{CloneFn, CloneFnTrait, CloneType};
use crate::element::{ElementPointer, Element, ElementMut, ElementRef};
use crate::any_vec_ptr::AnyVecPtr;
use crate::iter::{Iter, IterMut, IterRef};
use crate::mem::Mem;
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
/// Some operations return [`TempValue<Operation>`], which internally holds &mut to [`AnyVec`].
/// You can drop it, cast to concrete type, or put into another vector. (See [`AnyValue`])
///
/// *`Element: 'static` due to TypeId requirements*
pub struct AnyVec<Traits: ?Sized + Trait = dyn None, M: Mem = mem::Default>
{
    pub(crate) raw: AnyVecRaw<M>,
    clone_fn: <Traits as CloneType>::Type,
    phantom: PhantomData<Traits>
}

impl<Traits: ?Sized + Trait, M: Mem> AnyVec<Traits, M>
{
    /// Element should implement requested Traits
    ///
    /// `Mem::Builder` should be Default constructible.
    #[inline]
    pub fn new<T: 'static>() -> Self
    where
        T: SatisfyTraits<Traits>,
        M::Builder: Default
    {
        Self::new_in::<T>(Default::default())
    }

    /// Element should implement requested Traits
    #[inline]
    pub fn new_in<T: 'static>(mem_builder: M::Builder) -> Self
        where T: SatisfyTraits<Traits>
    {
        Self::with_capacity_in::<T>(0, mem_builder)
    }

    /// Element should implement requested Traits
    ///
    /// `Mem::Builder` should be Default constructible.
    #[inline]
    pub fn with_capacity<T: 'static>(capacity: usize) -> Self
    where
        T: SatisfyTraits<Traits>,
        M::Builder: Default
    {
        Self::with_capacity_in::<T>(capacity, Default::default())
    }

    /// Element should implement requested Traits
    pub fn with_capacity_in<T: 'static>(capacity: usize, mem_builder: M::Builder) -> Self
        where T: SatisfyTraits<Traits>
    {
        let clone_fn = <T as CloneFnTrait<Traits>>::CLONE_FN;
        Self{
            raw: AnyVecRaw::with_capacity_in::<T>(capacity, mem_builder),
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
    pub fn reserve(&mut self, additional: usize){
        self.raw.reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize){
        self.raw.reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self){
        self.raw.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize){
        self.raw.shrink_to(min_capacity)
    }

    #[inline]
    pub fn downcast_ref<Element: 'static>(&self) -> Option<AnyVecRef<Element, M>> {
        if self.element_typeid() == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<Element: 'static>(&self) -> AnyVecRef<Element, M> {
        AnyVecRef(AnyVecTyped::new(NonNull::from(&self.raw)))
    }

    #[inline]
    pub fn downcast_mut<Element: 'static>(&mut self) -> Option<AnyVecMut<Element, M>> {
        if self.element_typeid() == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<Element: 'static>(&mut self) -> AnyVecMut<Element, M> {
        AnyVecMut(AnyVecTyped::new(NonNull::from(&mut self.raw)))
    }

    #[inline]
    pub fn as_bytes(&self) -> *const u8 {
        self.raw.mem.as_ptr()
    }

    #[inline]
    pub fn iter(&self) -> IterRef<Traits, M>{
        Iter::new(AnyVecPtr::from(self), 0, self.len())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<Traits, M>{
        let len = self.len();
        Iter::new(AnyVecPtr::from(self), 0, len)
    }

    #[inline]
    unsafe fn get_element(&self, index: usize) -> ManuallyDrop<Element<Traits, M>>{
        let element = NonNull::new_unchecked(
            self.as_bytes().add(self.element_layout().size() * index) as *mut u8
        );
        ManuallyDrop::new(ElementPointer::new(
            AnyVecPtr::from(self),
            element
        ))
    }

    /// Return reference to element at `index` with bounds check.
    ///
    /// # Panics
    ///
    /// * Panics if index is out of bounds.
    #[inline]
    pub fn at(&self, index: usize) -> ElementRef<Traits, M>{
        self.get(index).unwrap()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<ElementRef<Traits, M>>{
        if index < self.len(){
            Some(unsafe{ self.get_unchecked(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> ElementRef<Traits, M>{
        ElementRef(self.get_element(index))
    }

    /// Return reference to element at `index` with bounds check.
    ///
    /// # Panics
    ///
    /// * Panics if index is out of bounds.
    #[inline]
    pub fn at_mut(&mut self, index: usize) -> ElementMut<Traits, M>{
        self.get_mut(index).unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<ElementMut<Traits, M>>{
        if index < self.len(){
            Some(unsafe{ self.get_unchecked_mut(index) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> ElementMut<Traits, M> {
        ElementMut(self.get_element(index))
    }

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if index is out of bounds.
    /// * Panics if out of memory.
    #[inline]
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
    pub fn remove(&mut self, index: usize) -> Remove<Traits, M> {
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
    pub fn swap_remove(&mut self, index: usize) -> SwapRemove<Traits, M> {
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
    #[inline]
    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Drain<Traits, M> {
        let Range{start, end} = into_range(self.len(), range);
        ops::Iter(drain::Drain::new(
            AnyVecPtr::from(self),
            start,
            end
        ))
    }

    /// Creates a splicing iterator that replaces the specified range in the vector
    /// with the given `replace_with` iterator and yields the removed items.
    /// `replace_with` does not need to be the same length as `range`.
    ///
    /// `range` is removed even if the iterator is not consumed until the end.
    ///
    /// The returned iterator keeps a mutable borrow on the vector.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements with indices in and past the range.
    ///
    /// [`mem::forget`]: std::mem::forget
    ///
    #[inline]
    pub fn splice<I: IntoIterator>(&mut self, range: impl RangeBounds<usize>, replace_with: I)
        -> Splice<Traits, M, I::IntoIter>
    where
        I::IntoIter: ExactSizeIterator,
        I::Item: AnyValue
    {
        let Range{start, end} = into_range(self.len(), range);
        ops::Iter(splice::Splice::new(
            AnyVecPtr::from(self),
            start,
            end,
            replace_with.into_iter()
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

unsafe impl<Traits: ?Sized + Send + Trait, M: Mem> Send for AnyVec<Traits, M> {}
unsafe impl<Traits: ?Sized + Sync + Trait, M: Mem> Sync for AnyVec<Traits, M> {}
impl<Traits: ?Sized + Cloneable + Trait, M: Mem> Clone for AnyVec<Traits, M>
{
    fn clone(&self) -> Self {
        Self{
            raw: unsafe{ self.raw.clone(self.clone_fn()) },
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }
}

impl<'a, Traits: ?Sized + Trait, M: Mem> IntoIterator for &'a AnyVec<Traits, M>{
    type Item = ElementRef<'a, Traits, M>;
    type IntoIter = IterRef<'a, Traits, M>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Traits: ?Sized + Trait, M: Mem> IntoIterator for &'a mut AnyVec<Traits, M>{
    type Item = ElementMut<'a, Traits, M>;
    type IntoIter = IterMut<'a, Traits, M>;

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
pub struct AnyVecRef<'a, T: 'static, M: Mem + 'a>(pub(crate) AnyVecTyped<'a, T, M>);
impl<'a, T: 'static, M: Mem + 'a> Clone for AnyVecRef<'a, T, M>{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, T: 'static, M: Mem + 'a> Deref for AnyVecRef<'a, T, M>{
    type Target = AnyVecTyped<'a, T, M>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, T: 'static, M: Mem + 'a> IntoIterator for AnyVecRef<'a, T, M>{
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
pub struct AnyVecMut<'a, T: 'static, M: Mem + 'a>(pub(crate) AnyVecTyped<'a, T, M>);
impl<'a, T: 'static, M: Mem + 'a> Deref for AnyVecMut<'a, T, M>{
    type Target = AnyVecTyped<'a, T, M>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, T: 'static, M: Mem + 'a> DerefMut for AnyVecMut<'a, T, M>{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<'a, T: 'static, M: Mem + 'a> IntoIterator for AnyVecMut<'a, T, M>{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        self.iter_mut()
    }
}