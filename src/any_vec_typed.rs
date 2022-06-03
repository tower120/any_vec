use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValueWrapper};
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{AnyValueTemp, Remove, SwapRemove};

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
        AnyValueTemp(Remove::<T>{
            any_vec: self.this_mut(),
            index,
            phantom: PhantomData
        }).downcast::<T>()
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        AnyValueTemp(SwapRemove::<T>{
            any_vec: self.this_mut(),
            index,
            phantom: PhantomData
        }).downcast::<T>()
    }

    #[inline]
    pub fn clear(&mut self){
        self.this_mut().clear();
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        unsafe{
            self.this().as_slice_unchecked::<T>()
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &'a mut[T] {
        unsafe{
            self.this_mut().as_mut_slice_unchecked::<T>()
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.this().len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.this().capacity()
    }
}