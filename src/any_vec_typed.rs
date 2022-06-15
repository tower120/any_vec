use std::iter::{FusedIterator};
use std::marker::PhantomData;
use std::ops::{Range, RangeBounds};
use std::ptr::NonNull;
use std::slice;
use crate::any_value::{AnyValue, AnyValueWrapper};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{ElementIter, remove, swap_remove, TempValue};
use crate::any_vec_ptr::AnyVecRawPtr;
use crate::into_range;
use crate::iter::ElementIterator;
use crate::ops::drain::Drain;
use crate::ops::splice::Splice;

/// Concrete type [`AnyVec`] representation.
///
/// Created with [`AnyVec::downcast_`]-family.
/// Accessed through [`AnyVecRef<T>`] or [`AnyVecMut<T>`]
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::downcast_`]: crate::AnyVec::downcast_ref
/// [`AnyVecRef<T>`]: crate::AnyVecRef
/// [`AnyVecMut<T>`]: crate::AnyVecMut
pub struct AnyVecTyped<'a, T: 'static>{
    // NonNull - to have one struct for both & and &mut
    any_vec: NonNull<AnyVecRaw>,
    phantom: PhantomData<&'a mut T>
}

unsafe impl<'a, T: 'static + Send> Send for AnyVecTyped<'a, T> {}
unsafe impl<'a, T: 'static + Sync> Sync for AnyVecTyped<'a, T> {}

impl<'a, T: 'static> AnyVecTyped<'a, T>{
    /// # Safety
    ///
    /// Unsafe, because type not checked
    #[inline]
    pub(crate) unsafe fn new(any_vec: NonNull<AnyVecRaw>) -> Self {
        Self{any_vec, phantom: PhantomData}
    }

    #[inline]
    fn this(&self) -> &'a AnyVecRaw {
        unsafe{ self.any_vec.as_ref() }
    }

    #[inline]
    fn this_mut(&mut self) -> &'a mut AnyVecRaw {
        unsafe{ self.any_vec.as_mut() }
    }

    #[inline]
    pub fn insert(&mut self, index: usize, value: T){
        unsafe{
            self.this_mut().insert_unchecked(index, AnyValueWrapper::new(value));
        }
    }

    #[inline]
    pub fn push(&mut self, value: T){
        unsafe{
            self.this_mut().push_unchecked(AnyValueWrapper::new(value));
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.this().index_check(index);
        unsafe{
            TempValue::<_>::new(remove::Remove::new(
                AnyVecRawPtr::<T>::from(self.any_vec),
                index
            )).downcast_unchecked::<T>()
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.this().index_check(index);
        unsafe{
            TempValue::<_>::new(swap_remove::SwapRemove::new(
                AnyVecRawPtr::<T>::from(self.any_vec),
                index
            )).downcast_unchecked::<T>()
        }
    }

    #[inline]
    pub fn drain(&mut self, range: impl RangeBounds<usize>)
        -> impl ElementIterator<Item = T>
    {
        let Range{start, end} = into_range(self.len(), range);
        ElementIter(Drain::new(
            AnyVecRawPtr::<T>::from(self.any_vec),
            start,
            end
        )).map(|e| unsafe{
            e.downcast_unchecked::<T>()
        })
    }

    #[inline]
    pub fn splice<I>(&mut self, range: impl RangeBounds<usize>, replace_with: I)
        -> impl ElementIterator<Item = T>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let Range{start, end} = into_range(self.len(), range);
        let replace_with = replace_with.into_iter()
            .map(|e| AnyValueWrapper::new(e));

        ElementIter(Splice::new(
            AnyVecRawPtr::<T>::from(self.any_vec),
            start,
            end,
            replace_with
        )).map(|e| unsafe{
            e.downcast_unchecked::<T>()
        })
    }

    #[inline]
    pub fn clear(&mut self){
        self.this_mut().clear();
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'a, T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'a, T> {
        self.as_mut_slice().iter_mut()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&'a T> {
        self.as_slice().get(index)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &'a T {
        self.as_slice().get_unchecked(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&'a mut T>{
        self.as_mut_slice().get_mut(index)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &'a mut T {
        self.as_mut_slice().get_unchecked_mut(index)
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        unsafe{
            slice::from_raw_parts(
                self.this().mem.as_ptr().cast::<T>(),
                self.this().len,
            )
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &'a mut[T] {
        unsafe{
            slice::from_raw_parts_mut(
                self.this_mut().mem.as_ptr().cast::<T>(),
                self.this().len,
            )
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.this().len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.this().capacity()
    }
}