// TODO: Drain


use crate::any_value::{AnyValue, AnyValueMut, Unknown};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use crate::AnyVec;
use crate::element::{Element};
use crate::any_vec_ptr::{AnyVecRawPtr, IAnyVecPtr, IAnyVecRawPtr};
use crate::any_vec_raw::AnyVecRaw;
use crate::refs::Ref;
use crate::traits::Trait;

type ElementRef<'a, AnyVecPtr> = Ref<ManuallyDrop<Element<'a, AnyVecPtr>>>;

struct DrainFilter<'a, AnyVecPtr: IAnyVecRawPtr, Filter: FnMut(ElementRef<'a, AnyVecPtr>) -> bool, T: 'static>
{
    any_vec_ptr: AnyVecPtr,
    start: usize,
    current: usize,
    end: usize,
    filter: Filter,
    phantom: PhantomData<&'a mut T>
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, Filter: FnMut(ElementRef<'a, AnyVecPtr>) -> bool, T: 'static>
    DrainFilter<'a, AnyVecPtr, Filter, T>
{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, start: usize, end: usize, filter: Filter) -> Self {
        Self{
            any_vec_ptr,
            start,
            current: start,
            end,
            filter,
            phantom: PhantomData
        }
    }

    #[inline]
    fn any_vec_raw(&self) -> &AnyVecRaw{
        unsafe { self.any_vec_ptr.any_vec_raw().as_ref() }
    }

    #[inline]
    unsafe fn ptr_at(&mut self, index: usize) -> NonNull<u8>{
        NonNull::new_unchecked(
            if Unknown::is::<T>(){
                self.any_vec_raw().mem.as_ptr()
                    .add(self.any_vec_raw().element_layout().size() * index)
            } else {
                self.any_vec_raw().mem.as_ptr().cast::<T>()
                    .add(index) as *mut u8
            }
        )
    }

    #[inline]
    unsafe fn element_at(&mut self, index: usize) -> Element<'a, AnyVecPtr> {
        Element::new(self.any_vec_ptr, self.ptr_at(index))
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, Filter: FnMut(ElementRef<'a, AnyVecPtr>) -> bool, T: 'static> Iterator
    for DrainFilter<'a, AnyVecPtr, Filter, T>
{
    type Item = Element<'a, AnyVecPtr>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // filter forward
        let mut element_ptr;
        loop{
            if self.current == self.end{
                return None;
            }

            let element_ref = unsafe{
                element_ptr = self.ptr_at(self.current);
                Ref(ManuallyDrop::new(
                    Element::new(self.any_vec_ptr, element_ptr)
                ))
            };
            if (self.filter)(element_ref) {
                break;
            }

            self.current += 1;
        }

        self.current += 1;
        Some(Element::new(self.any_vec_ptr, element_ptr))
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, Filter: FnMut(ElementRef<'a, AnyVecPtr>) -> bool, T: 'static>
    Drop for DrainFilter<'a, AnyVecPtr, Filter, T>
{
    fn drop(&mut self) {
        todo!()
    }
}