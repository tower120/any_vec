use std::mem::size_of;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;
use crate::AnyVec;

pub struct AnyVecTyped<'a, T: 'static>{
    any_vec: *mut AnyVec,       // TODO: NonNUll !!
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
    pub(crate) unsafe fn new(any_vec: *mut AnyVec) -> Self {
        Self{any_vec, phantom: PhantomData}
    }

    #[inline]
    fn this(&self) -> &AnyVec{
        unsafe{ &*self.any_vec }
    }

    #[inline]
    fn this_mut(&mut self) -> &mut AnyVec{
        unsafe{ &mut *self.any_vec }
    }

    #[inline]
    pub fn push(&mut self, value: T){
        unsafe{
            ptr::write(
                self.this_mut().push_uninit().as_mut_ptr() as *mut T,
                value
            );
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let mut out = MaybeUninit::<T>::uninit();
        unsafe{
            self.this_mut().swap_take_bytes_impl(
                index,
                size_of::<T>(),
                out.as_mut_ptr() as *mut u8);
            out.assume_init()
        }
    }

    #[inline]
    pub fn clear(&mut self){
        self.this_mut().clear();
    }

    #[inline]
    pub fn as_slice(&mut self) -> &[T] {
        unsafe{
            self.this_mut().as_slice_unchecked::<T>()
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