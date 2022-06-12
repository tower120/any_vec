use crate::any_value::{AnyValue, AnyValueMut, Unknown};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::{mem, ptr};
use std::ptr::NonNull;
use std::thread::current;
use crate::AnyVec;
use crate::element::{Element};
use crate::any_vec_ptr::{AnyVecRawPtr, IAnyVecPtr, IAnyVecRawPtr};
use crate::any_vec_raw::AnyVecRaw;
use crate::iter::Iter;
use crate::refs::Ref;
use crate::traits::Trait;

struct Drain<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static>
{
    any_vec_ptr: AnyVecPtr,
    start: usize,
    current: usize,
    end: usize,
    original_len: usize,
    phantom: PhantomData<&'a mut T>
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static> Drain<'a, AnyVecPtr, T>
{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        let any_vec_raw = unsafe{ any_vec_ptr.any_vec_raw().as_mut() };
        let original_len = any_vec_raw.len;

        // mem::forget and element drop panic "safety".
        any_vec_raw.len = start;

        Self{
            any_vec_ptr,
            start,
            current: start,
            end,
            original_len,
            phantom: PhantomData
        }
    }

    #[inline]
    fn any_vec_raw(&self) -> &AnyVecRaw{
        unsafe { self.any_vec_ptr.any_vec_raw().as_ref() }
    }

    #[inline]
    fn any_vec_raw_mut(&mut self) -> &mut AnyVecRaw{
        unsafe { self.any_vec_ptr.any_vec_raw().as_mut() }
    }

    #[inline]
    fn ptr_at(&self, index: usize) -> *mut u8 {
    unsafe{
        if Unknown::is::<T>(){
            self.any_vec_raw().mem.as_ptr()
                .add(self.any_vec_raw().element_layout().size() * index)
        } else {
            self.any_vec_raw().mem.as_ptr().cast::<T>()
                .add(index) as *mut u8
        }
    }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static> Iterator
    for Drain<'a, AnyVecPtr, T>
{
    type Item = Element<'a, AnyVecPtr>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end{
            return None;
        }

        let element = unsafe{
            let element_ptr = NonNull::new_unchecked(
                self.ptr_at(self.current)
            );
            Element::new(self.any_vec_ptr, element_ptr)
        };

        self.current += 1;
        Some(element)
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, T: 'static> Drop for Drain<'a, AnyVecPtr, T>
{
    fn drop(&mut self) {
        // 1. drop the rest of the elements
        if Unknown::is::<T>(){
             if let Some(drop_fn) = self.any_vec_raw().drop_fn(){
                 (drop_fn)(self.ptr_at(self.current), self.end - self.current);
            }
        } else if mem::needs_drop::<T>(){
            for index in self.current..self.end{
                unsafe{
                    ptr::drop_in_place(self.ptr_at(index) as *mut T);
                }
            }
        }

        // 2. mem move
        let distance = self.end - self.start;
        unsafe{
            ptr::copy(
                self.ptr_at(self.end),
                self.ptr_at(self.start),
                distance
            );
        }

        // 3. len
        self.any_vec_raw_mut().len = self.original_len - distance;
    }
}