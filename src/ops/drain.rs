use crate::any_value::{Unknown};
use std::{mem, ptr};
use std::cmp::{min};
use std::iter::{FusedIterator};
use crate::element::{Element};
use crate::any_vec_ptr::IAnyVecRawPtr;
use crate::any_vec_raw::AnyVecRaw;
use crate::iter::Iter;

pub struct Drain<'a, AnyVecPtr: IAnyVecRawPtr>
{
    iter: Iter<'a, AnyVecPtr>,
    start: usize,
    original_len: usize
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> Drain<'a, AnyVecPtr>
{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        let any_vec_raw = unsafe{ any_vec_ptr.any_vec_raw().as_mut() };
        let original_len = any_vec_raw.len;

        // mem::forget and element drop panic "safety".
        any_vec_raw.len = start;

        Self{
            iter: Iter::new(any_vec_ptr, start, end),
            start,
            original_len
        }
    }

    #[inline]
    fn any_vec_raw(&self) -> &AnyVecRaw{
        unsafe { self.iter.any_vec_ptr.any_vec_raw().as_ref() }
    }

    #[inline]
    fn any_vec_raw_mut(&mut self) -> &mut AnyVecRaw{
        unsafe { self.iter.any_vec_ptr.any_vec_raw().as_mut() }
    }

    #[inline]
    fn ptr_at(&self, index: usize) -> *mut u8 {
    unsafe{
        if Unknown::is::<AnyVecPtr::Element>(){
            self.any_vec_raw().mem.as_ptr()
                .add(self.any_vec_raw().element_layout().size() * index)
        } else {
            self.any_vec_raw().mem.as_ptr().cast::<AnyVecPtr::Element>()
                .add(index) as *mut u8
        }
    }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> Iterator
    for Drain<'a, AnyVecPtr>
{
    type Item = Element<'a, AnyVecPtr>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<'a, AnyVecPtr: IAnyVecRawPtr> ExactSizeIterator
    for Drain<'a, AnyVecPtr>
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
impl<'a, AnyVecPtr: IAnyVecRawPtr> FusedIterator
    for Drain<'a, AnyVecPtr>
{}


impl<'a, AnyVecPtr: IAnyVecRawPtr> Drop for Drain<'a, AnyVecPtr>
{
    fn drop(&mut self) {
        // 1. drop the rest of the elements
        if Unknown::is::<AnyVecPtr::Element>(){
             if let Some(drop_fn) = self.any_vec_raw().drop_fn(){
                 (drop_fn)(self.ptr_at(self.iter.index), self.iter.end - self.iter.index);
            }
        } else if mem::needs_drop::<AnyVecPtr::Element>(){
            for index in self.iter.index..self.iter.end{
                unsafe{
                    ptr::drop_in_place(self.ptr_at(index) as *mut AnyVecPtr::Element);
                }
            }
        }

        // 2. mem move
        let distance = self.iter.end - self.start;
        unsafe{
            let elements_left = self.original_len - self.iter.end;
            let copy_count = min(distance, elements_left);

            let src = self.ptr_at(self.iter.end);
            let dst = self.ptr_at(self.start);
            if Unknown::is::<AnyVecPtr::Element>(){
                ptr::copy(
                    src,
                    dst,
                    self.any_vec_raw().element_layout().size() * copy_count
                );
            } else {
                ptr::copy(
                    src as * const AnyVecPtr::Element,
                    dst as *mut AnyVecPtr::Element,
                    copy_count
                );
            }
        }

        // 3. len
        self.any_vec_raw_mut().len = self.original_len - distance;
    }
}