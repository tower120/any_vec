use core::marker::PhantomData;
use core::{fmt, mem, slice};
use core::fmt::{Debug, Formatter};
use core::mem::MaybeUninit;
use core::ops::{Range, RangeBounds};
use core::ptr::{addr_of, NonNull};
use crate::any_value::{AnyValueSizeless, AnyValueWrapper, Unknown};
use crate::any_vec_ptr::AnyVecRawPtr;
use crate::any_vec_raw::AnyVecRaw;
use crate::{into_range, ElementIterator};
use crate::mem::{Mem, MemBuilder, MemResizable};
use crate::ops::{pop, remove, swap_remove, Iter, TempValue};
use crate::ops::drain::Drain;
use crate::ops::splice::Splice;

/// Concrete type [`AnyVec`] representation.
/// 
/// Can only be obtained as `&`/`&mut` with [`downcast_ref`]/[`downcast_mut`]. 
///
/// Operations with concrete type are somewhat faster, due to
/// the fact, that compiler are able to optimize harder with full
/// type knowledge.
///
/// [`AnyVec`]: crate::AnyVec
/// [`downcast_ref`]: crate::AnyVec::downcast_ref
/// [`downcast_mut`]: crate::AnyVec::downcast_mut 
#[repr(transparent)]
pub struct AnyVecTyped<T: 'static, M: MemBuilder>(pub(crate) PhantomData<(T, M)>);

unsafe impl<T: 'static + Send, M: MemBuilder + Send> Send for AnyVecTyped<T, M>
    where M::Mem: Send
{}
unsafe impl<T: 'static + Sync, M: MemBuilder + Sync> Sync for AnyVecTyped<T, M>
    where M::Mem: Sync
{}

impl<T: 'static, M: MemBuilder> AnyVecTyped<T, M>{
    #[inline]
    fn this(&self) -> &AnyVecRaw<M> {
         unsafe{
            // let offset = mem::offset_of!(AnyVecRaw<M>, any_vec_typed);
            //let t: &AnyVecTyped<Unknown, M> = mem::transmute(self);
             
            let self_ptr = self as *const _ as *const u8;
            let ptr = self_ptr.sub(mem::offset_of!(AnyVecRaw<M>, any_vec_typed));
             
             NonNull::new_unchecked(ptr as *mut _).as_ref()
             // ptr.cast::<AnyVecRaw<M>>().as_ref().unwrap()
             // &*(ptr as *const AnyVecRaw<M>)
            //&*ptr.cast()
        }
    }
    
    #[inline]
    fn this_mut(&mut self) -> &mut AnyVecRaw<M> {
         unsafe{
            let offset = mem::offset_of!(AnyVecRaw<M>, any_vec_typed);
            let self_ptr = self as *mut Self as *mut u8;
            let ptr = self_ptr.offset(-(offset as isize));
            &mut *ptr.cast()
        }
    }
    
    #[inline]
    pub fn reserve(&mut self, additional: usize)
        where M::Mem: MemResizable
    {
        self.this_mut().reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize)
        where M::Mem: MemResizable
    {
        self.this_mut().reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self)
        where M::Mem: MemResizable
    {
        self.this_mut().shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize)
        where M::Mem: MemResizable
    {
        self.this_mut().shrink_to(min_capacity)
    }
    
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.this_mut().set_len(new_len);
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
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty(){
            None
        } else {
            let value = unsafe{
                TempValue::new(pop::Pop::new(
                    AnyVecRawPtr::<T, M>::from(self.this_mut())
                )).downcast_unchecked::<T>()
            };
            Some(value)
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.this().index_check(index);
        unsafe{
            TempValue::new(remove::Remove::new(
                AnyVecRawPtr::<T, M>::from(self.this_mut()),
                index
            )).downcast_unchecked::<T>()
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.this().index_check(index);
        unsafe{
            TempValue::new(swap_remove::SwapRemove::new(
                AnyVecRawPtr::<T, M>::from(self.this_mut()),
                index
            )).downcast_unchecked::<T>()
        }
    }
    
    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    /// 
    /// # Panics
    /// 
    /// * Panics if out of memory.
    pub fn append<OtherM: MemBuilder>(
        &mut self, other: &mut AnyVecTyped<T, OtherM>
    ) {
        unsafe{
            self.this_mut().append_unchecked::<T, _>(other.this_mut());
        }
    }    

    #[inline]
    pub fn drain(&mut self, range: impl RangeBounds<usize>)
        -> impl ElementIterator<Item = T> + '_
    {
        let Range{start, end} = into_range(self.len(), range);
        Iter(Drain::new(
            AnyVecRawPtr::<T, M>::from(self.this_mut()),
            start,
            end
        )).map(|e| unsafe{
            e.downcast_unchecked::<T>()
        })
    }  
    
    #[inline]
    pub fn splice<'a, I>(&'a mut self, range: impl RangeBounds<usize>, replace_with: I)
        -> impl ElementIterator<Item = T> + 'a
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator + 'a,
    {
        let Range{start, end} = into_range(self.len(), range);
        let replace_with = replace_with.into_iter()
            .map(|e| AnyValueWrapper::new(e));

        Iter(Splice::new(
            AnyVecRawPtr::<T, M>::from(self.this_mut()),
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
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
    
    #[inline]
    pub fn at(&self, index: usize) -> &T{
        self.get(index).unwrap()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.as_slice().get_unchecked(index)
    }

    #[inline]
    pub fn at_mut(&mut self, index: usize) -> &mut T{
        self.get_mut(index).unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T>{
        self.as_mut_slice().get_mut(index)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.as_mut_slice().get_unchecked_mut(index)
    }    
    
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.this().mem.as_ptr().cast::<T>()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.this_mut().mem.as_mut_ptr().cast::<T>()
    }
    
    
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe{
            slice::from_raw_parts(
                self.as_ptr(),
                self.len(),
            )
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut[T] {
        unsafe{
            slice::from_raw_parts_mut(
                self.as_mut_ptr(),
                self.len(),
            )
        }
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut[MaybeUninit<T>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len()) as *mut MaybeUninit<T>,
                self.capacity() - self.len(),
            )
        }
    }    
    
    #[inline]
    pub fn len(&self) -> usize {
        self.this().len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.this().capacity()
    }
}

impl<T: 'static + Debug, M: MemBuilder> Debug for AnyVecTyped<T, M>{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        (*self.as_slice()).fmt(f)
    }
}

impl<'a, T: 'static, M: MemBuilder> IntoIterator for &'a AnyVecTyped<T, M>{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: 'static, M: MemBuilder> IntoIterator for &'a mut AnyVecTyped<T, M>{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}