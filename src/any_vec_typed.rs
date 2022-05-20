use std::mem::size_of;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;
use std::ptr::NonNull;
use crate::{AnyValue, AnyValueWrapper, AnyVec};

/// Concrete type [`AnyVec`] representation.
pub struct AnyVecTyped<'a, T: 'static>{
    any_vec: NonNull<AnyVec>,
    phantom: PhantomData<&'a mut T>
}

unsafe impl<'a, T: 'static> Send for AnyVecTyped<'a, T>
    where T: Send
{}

unsafe impl<'a, T: 'static> Sync for AnyVecTyped<'a, T>
    where T: Sync
{}

impl<'a, T: 'static> AnyVecTyped<'a, T>{
    /// # Safety
    ///
    /// Unsafe, because type not checked
    #[inline]
    pub(crate) unsafe fn new(any_vec: NonNull<AnyVec>) -> Self {
        Self{any_vec, phantom: PhantomData}
    }

    #[inline]
    fn this(&self) -> &AnyVec{
        unsafe{ self.any_vec.as_ref() }
    }

    #[inline]
    fn this_mut(&mut self) -> &mut AnyVec{
        unsafe{ self.any_vec.as_mut() }
    }

    #[inline]
    pub fn insert(&mut self, index: usize, value: T){
        unsafe{
            ptr::write(
                self.this_mut().insert_uninit(index).as_mut_ptr() as *mut T,
                value
            );
        }
    }

    #[inline]
    pub fn push(&mut self, value: T){
        self.this_mut().push(AnyValueWrapper::new(value));
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        let mut out = MaybeUninit::<T>::uninit();
        unsafe{
            self.this_mut().remove_into_impl(
                index,
                size_of::<T>(),
                out.as_mut_ptr() as *mut u8);
            out.assume_init()
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let mut out = MaybeUninit::<T>::uninit();
        unsafe{
            self.this_mut().swap_remove_into_impl(
                index,
                size_of::<T>(),
                out.as_mut_ptr() as *mut u8);
            out.assume_init()
        }
    }

    #[inline]
    pub fn swap_remove_v2(&mut self, index: usize) -> T {
        self.this_mut().swap_remove_v3(index).downcast::<T>()
    }

    #[inline]
    pub fn clear(&mut self){
        self.this_mut().clear();
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe{
            self.this().as_slice_unchecked::<T>()
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut[T] {
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