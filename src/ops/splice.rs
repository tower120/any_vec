use crate::any_vec_ptr::{AnyVecPtr, AnyVecRawPtr, IAnyVecRawPtr};
use crate::{any_vec_ptr, Iter};
use crate::any_value::AnyValue;
use crate::ops::iter::Iterable;
use crate::traits::Trait;

pub struct Splice<'a, AnyVecPtr: IAnyVecRawPtr, ReplaceIter: ExactSizeIterator>
where
    ReplaceIter::Item: AnyValue
{
    iter: Iter<'a, AnyVecPtr>,
    start: usize,
    original_len: usize,
    replace_with: ReplaceIter
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, ReplaceIter: ExactSizeIterator>
    Splice<'a, AnyVecPtr, ReplaceIter>
where
    ReplaceIter::Item: AnyValue
{
    #[inline]
    pub fn new(
        any_vec_ptr: AnyVecPtr, start: usize, end: usize,
        replace_with: ReplaceIter
    ) -> Self {
        debug_assert!(start <= end);
        let any_vec_raw = unsafe{ any_vec_ptr.any_vec_raw().as_mut() };
        let original_len = any_vec_raw.len;
        debug_assert!(end <= original_len);

        // mem::forget and element drop panic "safety".
        any_vec_raw.len = start;

        Self{
            iter: Iter::new(any_vec_ptr, start, end),
            start,
            original_len,
            replace_with
        }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, ReplaceIter: ExactSizeIterator> Iterable
for
    Splice<'a, AnyVecPtr, ReplaceIter>
where
    ReplaceIter::Item: AnyValue
{
    type Iter = Iter<'a, AnyVecPtr>;

    #[inline]
    fn iter(&self) -> &Self::Iter {
        &self.iter
    }

    #[inline]
    fn iter_mut(&mut self) -> &mut Self::Iter {
        &mut self.iter
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, ReplaceIter: ExactSizeIterator> Drop
for
    Splice<'a, AnyVecPtr, ReplaceIter>
where
    ReplaceIter::Item: AnyValue
{
    fn drop(&mut self) {
        use any_vec_ptr::utils::*;
        let any_vec_ptr = self.iter.any_vec_ptr;

        let elements_left = self.original_len - self.iter.end;
        let replace_end = self.start + self.replace_with.len();
        let new_len = replace_end + elements_left;

        // 0. capacity.
        {
            let any_vec_raw = unsafe{any_vec_ptr.any_vec_raw().as_mut()};
            if new_len > any_vec_raw.capacity(){
                any_vec_raw.grow();
            }
        }

        // 1. drop elements.
        unsafe{
            drop_elements_range(
                any_vec_ptr,
                self.iter.index,
                self.iter.end
            );
        }

        // 2. move elements
        unsafe{
            move_elements_at(
                any_vec_ptr,
                self.iter.end,
                replace_end,
                elements_left
            );
        }

        // 3. move replace_with in
        unsafe{
            let element_size = element_size(any_vec_ptr);
            let mut ptr = element_ptr_at(any_vec_ptr, self.start);
            while let Some(replace_element) = self.replace_with.next() {
                replace_element.move_into(ptr);
                ptr = ptr.add(element_size);
            }
        }

        // 4. restore len
        {
            let any_vec_raw = unsafe{any_vec_ptr.any_vec_raw().as_mut()};
            any_vec_raw.len = new_len;
        }
    }
}

#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, Traits: ?Sized + Send + Trait, ReplaceIter: ExactSizeIterator> Send
for
    Splice<'a, AnyVecPtr<Traits>, ReplaceIter>
where
    ReplaceIter::Item: AnyValue + Send
{}

#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, Type: Send, ReplaceIter: ExactSizeIterator> Send
for
    Splice<'a, AnyVecRawPtr<Type>, ReplaceIter>
where
    ReplaceIter::Item: AnyValue + Send
{}


#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, Traits: ?Sized + Sync + Trait, ReplaceIter: ExactSizeIterator> Sync
for
    Splice<'a, AnyVecPtr<Traits>, ReplaceIter>
where
    ReplaceIter::Item: AnyValue + Sync
{}

#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, Type: Sync, ReplaceIter: ExactSizeIterator> Sync
for
    Splice<'a, AnyVecRawPtr<Type>, ReplaceIter>
where
    ReplaceIter::Item: AnyValue + Sync
{}
