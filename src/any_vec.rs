use std::alloc::Layout;
use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut, Range, RangeBounds};
use std::ptr::NonNull;
use std::slice;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use crate::{AnyVecTyped, into_range, mem, ops};
use crate::any_value::{AnyValue};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{TempValue, Remove, SwapRemove, remove, swap_remove, Pop, pop};
use crate::ops::{Drain, Splice, drain, splice};
use crate::any_vec::traits::{None};
use crate::clone_type::{CloneFn, CloneFnTrait, CloneType};
use crate::element::{ElementPointer, ElementMut, ElementRef};
use crate::any_vec_ptr::AnyVecPtr;
use crate::iter::{Iter, IterMut, IterRef};
use crate::mem::{Mem, MemBuilder, MemBuilderSizeable, MemResizable};
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
    pub trait Trait: 'static + crate::clone_type::CloneType{}
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
/// Only destruct and clone operations have indirect call overhead.
///
/// You can make AnyVec [`Send`]-able, [`Sync`]-able, [`Cloneable`], by
/// specifying trait constraints: `AnyVec<dyn Cloneable + Sync + Send>`. See [`traits`].
///
/// Some operations return [`TempValue<Operation>`], which internally holds &mut to [`AnyVec`].
/// You can drop it, cast to concrete type, or put into another vector. (See [`AnyValue`])
///
/// *`Element: 'static` due to TypeId requirements*
pub struct AnyVec<Traits: ?Sized + Trait = dyn None, M: MemBuilder = mem::Default>
{
    pub(crate) raw: AnyVecRaw<M>,
    clone_fn: <Traits as CloneType>::Type,  // ZST if Traits: !Cloneable
    phantom: PhantomData<Traits>
}

impl<Traits: ?Sized + Trait, M: MemBuilder> AnyVec<Traits, M>
{
    #[inline]
    fn build<T: SatisfyTraits<Traits>>(raw: AnyVecRaw<M>) -> Self {
        let clone_fn = <T as CloneFnTrait<Traits>>::CLONE_FN;
        Self{
            raw,
            clone_fn: <Traits as CloneType>::new(clone_fn),
            phantom: PhantomData
        }
    }

    /// Constructs empty [`AnyVec`] with elements of type `T`,
    /// using [`Default`] [`MemBuilder`].
    ///
    /// `T` should satisfy requested Traits.
    ///
    /// Not available, if provided [`MemBuilder`] is not [`Default`].
    #[inline]
    pub fn new<T: 'static>() -> Self
    where
        T: SatisfyTraits<Traits>,
        M: Default
    {
        Self::new_in::<T>(Default::default())
    }

    /// Constructs empty [`AnyVec`] with elements of type `T`,
    /// using provided `mem_builder`.
    ///
    /// `T` should satisfy requested Traits.
    #[inline]
    pub fn new_in<T: 'static>(mut mem_builder: M) -> Self
        where T: SatisfyTraits<Traits>
    {
        let mem = mem_builder.build(Layout::new::<T>());
        let raw = AnyVecRaw::new::<T>(mem_builder, mem);
        Self::build::<T>(raw)
    }

    /// Constructs empty [`AnyVec`] with specified capacity and
    /// elements of type `T`, using [`Default`] [`MemBuilder`].
    ///
    /// `T` should satisfy requested Traits.
    ///
    /// Not available, if provided [`MemBuilder`] is not
    /// [`MemBuilderSizeable`] and [`Default`].
    #[inline]
    pub fn with_capacity<T: 'static>(capacity: usize) -> Self
    where
        T: SatisfyTraits<Traits>,
        M: MemBuilderSizeable,
        M: Default
    {
        Self::with_capacity_in::<T>(capacity, Default::default())
    }

    /// Constructs empty [`AnyVec`] with specified capacity and
    /// elements of type `T`, using `mem_builder`.
    ///
    /// `T` should satisfy requested Traits.
    ///
    /// Not available, if provided [`MemBuilder`] is not
    /// [`MemBuilderSizeable`].
    pub fn with_capacity_in<T: 'static>(capacity: usize, mut mem_builder: M) -> Self
    where
        T: SatisfyTraits<Traits>,
        M: MemBuilderSizeable
    {
        let mem = mem_builder.build_with_size(Layout::new::<T>(), capacity);
        let raw = AnyVecRaw::new::<T>(mem_builder, mem);
        Self::build::<T>(raw)
    }

    /// Constructs **empty** [`AnyVec`] with the same elements type, `Traits` and `MemBuilder`.
    /// IOW, same as [`clone`], but without elements copy.
    ///
    /// [`clone`]: Clone::clone
    #[inline]
    pub fn clone_empty(&self) -> Self {
        Self {
            raw: self.raw.clone_empty(),
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }

    /// Constructs **empty** [`AnyVec`] with the same elements type and `Traits`,
    /// but with other `MemBuilder`.
    ///
    /// Use it to construct intermediate storage, with fast [`MemBuilder`].
    ///
    /// # Example
    ///
    /// ```
    /// # use any_vec::any_value::AnyValueCloneable;
    /// # use any_vec::AnyVec;
    /// # use any_vec::mem::Stack;
    /// # use any_vec::traits::Cloneable;
    /// # let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    /// # any_vec.downcast_mut::<String>().unwrap().push(String::from("0"));
    /// let mut tmp = any_vec.clone_empty_in(Stack::<256>);
    ///     tmp.push(any_vec.at(0).lazy_clone());
    /// any_vec.push(tmp.pop().unwrap());
    /// ```
    #[inline]
    pub fn clone_empty_in<NewM: MemBuilder>(&self, mem_builder: NewM) -> AnyVec<Traits, NewM> {
        AnyVec {
            raw: self.raw.clone_empty_in(mem_builder),
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }

    #[inline]
    pub(crate) fn clone_fn(&self) -> CloneFn{
        <Traits as CloneType>::get(self.clone_fn)
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given container. More space may be reserved to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Exact behavior defined by
    /// implementation of [`MemResizable`]. Does nothing if capacity is already sufficient.
    ///
    /// Not available, if provided [`MemBuilder::Mem`] is not [`MemResizable`].
    ///
    /// # Panics
    ///
    /// [`MemResizable`] implementation may panic - see implementation description.
    #[inline]
    pub fn reserve(&mut self, additional: usize)
        where M::Mem: MemResizable
    {
        self.raw.reserve(additional)
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the given container. After calling `reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Exact behavior defined by implementation of [`MemResizable`].
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the [`Mem`] implementation may grow bigger then requested.
    /// Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`reserve`] if future insertions are expected.
    ///
    /// Not available, if provided [`MemBuilder::Mem`] is not [`MemResizable`].
    ///
    /// # Panics
    ///
    /// [`MemResizable`] implementation may panic - see implementation description.
    ///
    /// [`reserve`]: Self::reserve
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize)
        where M::Mem: MemResizable
    {
        self.raw.reserve_exact(additional)
    }

    /// Shrinks the capacity as much as possible.
    /// Exact behavior defined by implementation of [`MemResizable`].
    ///
    /// Not available, if provided [`MemBuilder::Mem`] is not [`MemResizable`].
    ///
    /// # Panics
    ///
    /// [`MemResizable`] implementation may panic - see implementation description.
    #[inline]
    pub fn shrink_to_fit(&mut self)
        where M::Mem: MemResizable
    {
        self.raw.shrink_to_fit()
    }

    /// Shrinks the capacity of the vector with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length
    /// and the supplied value. Exact behavior defined by implementation of [`MemResizable`].
    ///
    /// If the current capacity is less than the lower limit, this is a no-op.
    ///
    /// Not available, if provided [`MemBuilder::Mem`] is not [`MemResizable`].
    ///
    /// # Panics
    ///
    /// [`MemResizable`] implementation may panic - see implementation description.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize)
        where M::Mem: MemResizable
    {
        self.raw.shrink_to(min_capacity)
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.raw.set_len(new_len);
    }

    /// Returns [`AnyVecRef`] - typed view to const AnyVec,
    /// if container holds elements of type T, or None if it isn’t.
    #[inline]
    pub fn downcast_ref<T: 'static>(&self) -> Option<AnyVecRef<T, M>> {
        if self.element_typeid() == TypeId::of::<T>() {
            unsafe{ Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    /// Returns [`AnyVecRef`] - typed view to const AnyVec.
    ///
    /// # Safety
    ///
    /// The container elements must be of type `T`.
    /// Calling this method with the incorrect type is undefined behavior.
    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> AnyVecRef<T, M> {
        AnyVecRef(AnyVecTyped::new(NonNull::from(&self.raw)))
    }

    /// Returns [`AnyVecMut`] - typed view to mut AnyVec,
    /// if container holds elements of type T, or None if it isn’t.
    #[inline]
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<AnyVecMut<T, M>> {
        if self.element_typeid() == TypeId::of::<T>() {
            unsafe{ Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    /// Returns [`AnyVecMut`] - typed view to mut AnyVec.
    ///
    /// # Safety
    ///
    /// The container elements must be of type `T`.
    /// Calling this method with the incorrect type is undefined behavior.
    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> AnyVecMut<T, M> {
        AnyVecMut(AnyVecTyped::new(NonNull::from(&mut self.raw)))
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe{from_raw_parts(
            self.raw.mem.as_ptr(),
            self.len() * self.element_layout().size()
        )}
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8]{
        unsafe{from_raw_parts_mut(
            self.raw.mem.as_mut_ptr(),
            self.len() * self.element_layout().size()
        )}
    }

    #[inline]
    pub fn spare_bytes_mut(&mut self) -> &mut [MaybeUninit<u8>]{
        unsafe{from_raw_parts_mut(
            self.raw.mem.as_mut_ptr().add(self.len()) as *mut MaybeUninit<u8>,
            (self.capacity() - self.len()) * self.element_layout().size()
        )}
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
        let element_ptr = self.raw.get_unchecked(index) as *mut u8;
        ElementRef(
            ManuallyDrop::new(ElementPointer::new(
                AnyVecPtr::from(self),
                NonNull::new_unchecked(element_ptr)
            ))
        )
    }

    /// Return mutable reference to element at `index` with bounds check.
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
        let element_ptr = self.raw.get_unchecked_mut(index);
        ElementMut(
            ManuallyDrop::new(ElementPointer::new(
                AnyVecPtr::from(self),
                NonNull::new_unchecked(element_ptr)
            ))
        )
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

    /// # Leaking
    ///
    /// If the returned [`TempValue`] goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector will lost and leak last element.
    ///
    /// [`mem::forget`]: std::mem::forget
    ///
    #[inline]
    pub fn pop(&mut self) -> Option<Pop<Traits, M>> {
        if self.is_empty(){
            None
        } else {
            Some(TempValue::new(
                pop::Pop::new(AnyVecPtr::from(self))
            ))
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
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }
}

unsafe impl<Traits: ?Sized + Send + Trait, M: MemBuilder + Send> Send for AnyVec<Traits, M>
    where M::Mem: Send
{}
unsafe impl<Traits: ?Sized + Sync + Trait, M: MemBuilder + Sync> Sync for AnyVec<Traits, M>
    where M::Mem: Sync
{}
impl<Traits: ?Sized + Cloneable + Trait, M: MemBuilder> Clone for AnyVec<Traits, M>
{
    fn clone(&self) -> Self {
        Self{
            raw: unsafe{ self.raw.clone(self.clone_fn()) },
            clone_fn: self.clone_fn,
            phantom: PhantomData
        }
    }
}

impl<Traits: ?Sized + Trait, M: MemBuilder> Debug for AnyVec<Traits, M>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyVec")
         .field("typeid", &self.element_typeid())
         .field("len", &self.len())
         .finish()
    }
}

impl<'a, Traits: ?Sized + Trait, M: MemBuilder> IntoIterator for &'a AnyVec<Traits, M>{
    type Item = ElementRef<'a, Traits, M>;
    type IntoIter = IterRef<'a, Traits, M>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Traits: ?Sized + Trait, M: MemBuilder> IntoIterator for &'a mut AnyVec<Traits, M>{
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
pub struct AnyVecRef<'a, T: 'static, M: MemBuilder + 'a>(pub(crate) AnyVecTyped<'a, T, M>);
impl<'a, T: 'static, M: MemBuilder + 'a> Clone for AnyVecRef<'a, T, M>{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, T: 'static, M: MemBuilder + 'a> Deref for AnyVecRef<'a, T, M>{
    type Target = AnyVecTyped<'a, T, M>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, T: 'static, M: MemBuilder + 'a> IntoIterator for AnyVecRef<'a, T, M>{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T: 'static + Debug, M: MemBuilder + 'a> Debug for AnyVecRef<'a, T, M>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Typed view to &mut [`AnyVec`].
///
/// You can get it from [`AnyVec::downcast_mut`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::downcast_mut`]: crate::AnyVec::downcast_mut
pub struct AnyVecMut<'a, T: 'static, M: MemBuilder + 'a>(pub(crate) AnyVecTyped<'a, T, M>);
impl<'a, T: 'static, M: MemBuilder + 'a> Deref for AnyVecMut<'a, T, M>{
    type Target = AnyVecTyped<'a, T, M>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, T: 'static, M: MemBuilder + 'a> DerefMut for AnyVecMut<'a, T, M>{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<'a, T: 'static, M: MemBuilder + 'a> IntoIterator for AnyVecMut<'a, T, M>{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<'a, T: 'static + Debug, M: MemBuilder + 'a> Debug for AnyVecMut<'a, T, M>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}