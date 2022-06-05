use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValueWrapper};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{remove, swap_remove, TempValue};
use crate::ops::any_vec_ptr::AnyVecRawPtr;
use crate::traits::EmptyTrait;

/// Concrete type [`AnyVec`] representation.
///
/// You can access it through [`AnyVecRef<T>`] or [`AnyVecMut<T>`]
///
/// [`AnyVec`]: crate::AnyVec
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
        self.this_mut().insert(index, AnyValueWrapper::new(value));
    }

    #[inline]
    pub fn push(&mut self, value: T){
        self.this_mut().push(AnyValueWrapper::new(value));
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.this().index_check(index);
        unsafe{
            TempValue::<_>::new(remove::Remove::<_, T>::new(
                AnyVecRawPtr::from(self.any_vec),
                index
            )).downcast_unchecked::<T>()
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.this().index_check(index);
        unsafe{
            TempValue::<_>::new(swap_remove::SwapRemove::<_, T>::new(
                AnyVecRawPtr::from(self.any_vec),
                index
            )).downcast_unchecked::<T>()
        }
    }

    #[inline]
    pub fn clear(&mut self){
        self.this_mut().clear();
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        unsafe{
            std::slice::from_raw_parts(
                self.this().mem.as_ptr().cast::<T>(),
                self.this().len,
            )
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &'a mut[T] {
        unsafe{
            std::slice::from_raw_parts_mut(
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