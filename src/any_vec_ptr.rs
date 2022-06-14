//! Type dispatched analog of `enum{*AnyVecRaw, *AnyVec<Traits>}`.

use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::any_value::Unknown;
use crate::any_vec_raw::AnyVecRaw;
use crate::AnyVec;
use crate::traits::Trait;

pub trait IAnyVecRawPtr: Copy{
    /// Known element type of AnyVec
    type Element: 'static/* = Unknown*/;
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw>;
}
pub trait IAnyVecPtr<Traits: ?Sized + Trait>: IAnyVecRawPtr{
    fn any_vec(&self) -> NonNull<AnyVec<Traits>>;
}


pub struct AnyVecRawPtr<Type: 'static>{
    ptr: NonNull<AnyVecRaw>,
    phantom: PhantomData<*mut Type>
}
impl<Type> From<NonNull<AnyVecRaw>> for AnyVecRawPtr<Type>{
    #[inline]
    fn from(ptr: NonNull<AnyVecRaw>) -> Self {
        Self{ptr, phantom: PhantomData}
    }
}
impl<Type> Copy for AnyVecRawPtr<Type> {}
impl<Type> Clone for AnyVecRawPtr<Type> {
    #[inline]
    fn clone(&self) -> Self {
        Self{
            ptr: self.ptr,
            phantom: PhantomData
        }
    }
}

impl<Type> IAnyVecRawPtr for AnyVecRawPtr<Type>{
    type Element = Type;

    #[inline]
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw> {
        self.ptr
    }
}


pub struct AnyVecPtr<Traits: ?Sized + Trait>{
    ptr: NonNull<AnyVec<Traits>>
}
impl<Traits: ?Sized + Trait> From<NonNull<AnyVec<Traits>>> for AnyVecPtr<Traits> {
    #[inline]
    fn from(ptr: NonNull<AnyVec<Traits>>) -> Self {
        Self{ptr}
    }
}
impl<Traits: ?Sized + Trait> From<&mut AnyVec<Traits>> for AnyVecPtr<Traits> {
    #[inline]
    fn from(reference: &mut AnyVec<Traits>) -> Self {
        Self{ptr: NonNull::from(reference)}
    }
}
impl<Traits: ?Sized + Trait> From<&AnyVec<Traits>> for AnyVecPtr<Traits> {
    #[inline]
    fn from(reference: &AnyVec<Traits>) -> Self {
        Self{ptr: NonNull::from(reference)}
    }
}
impl<Traits: ?Sized + Trait> Clone for AnyVecPtr<Traits>{
    #[inline]
    fn clone(&self) -> Self {
        Self{ptr: self.ptr}
    }
}
impl<Traits: ?Sized + Trait> Copy for AnyVecPtr<Traits>{}

impl<Traits: ?Sized + Trait> IAnyVecRawPtr for AnyVecPtr<Traits> {
    type Element = Unknown;

    #[inline]
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw> {
        NonNull::from(unsafe{&self.ptr.as_ref().raw})
    }
}
impl<Traits: ?Sized + Trait> IAnyVecPtr<Traits> for AnyVecPtr<Traits> {
    #[inline]
    fn any_vec(&self) -> NonNull<AnyVec<Traits>> {
        self.ptr
    }
}


/// Type knowledge optimized operations.
pub(crate) mod utils{
    use std::{mem, ptr};
    use std::mem::size_of;
    use crate::any_value::Unknown;
    use crate::any_vec_ptr::IAnyVecRawPtr;

    #[inline]
    pub fn element_size<AnyVecPtr: IAnyVecRawPtr>(any_vec_ptr: AnyVecPtr) -> usize
    {
        if Unknown::is::<AnyVecPtr::Element>(){
            let any_vec_raw = unsafe{ any_vec_ptr.any_vec_raw().as_ref() };
            any_vec_raw.element_layout().size()
        } else {
            size_of::<AnyVecPtr::Element>()
        }
    }

    #[inline]
    pub fn element_ptr_at<AnyVecPtr: IAnyVecRawPtr>(any_vec_ptr: AnyVecPtr, index: usize)
        -> *mut u8
    { unsafe{
        let any_vec_raw = any_vec_ptr.any_vec_raw().as_mut();

        if Unknown::is::<AnyVecPtr::Element>(){
            any_vec_raw.mem.as_ptr()
                .add(any_vec_raw.element_layout().size() * index)
        } else {
            any_vec_raw.mem.as_ptr().cast::<AnyVecPtr::Element>()
                .add(index) as *mut u8
        }
    } }

    #[inline]
    pub unsafe fn move_elements_at<AnyVecPtr: IAnyVecRawPtr>
        (any_vec_ptr: AnyVecPtr, src_index: usize, dst_index: usize, len: usize)
    {
        let src = element_ptr_at(any_vec_ptr, src_index);
        let dst = element_ptr_at(any_vec_ptr, dst_index);
        if Unknown::is::<AnyVecPtr::Element>(){
            let any_vec_raw = any_vec_ptr.any_vec_raw().as_ref();
            ptr::copy(
                src,
                dst,
                any_vec_raw.element_layout().size() * len
            );
        } else {
            ptr::copy(
                src as * const AnyVecPtr::Element,
                dst as *mut AnyVecPtr::Element,
                len
            );
        }
    }

    #[inline]
    pub unsafe fn drop_elements_range<AnyVecPtr: IAnyVecRawPtr>
        (any_vec_ptr: AnyVecPtr, start_index: usize, end_index: usize)
    {
        debug_assert!(start_index <= end_index);

        if Unknown::is::<AnyVecPtr::Element>(){
            let any_vec_raw = any_vec_ptr.any_vec_raw().as_ref();
            if let Some(drop_fn) = any_vec_raw.drop_fn(){
                (drop_fn)(
                    element_ptr_at(any_vec_ptr, start_index),
                    end_index - start_index
                );
            }
        } else if mem::needs_drop::<AnyVecPtr::Element>(){
            // drop as slice. This is marginally faster then one by one.
            let start_ptr = element_ptr_at(any_vec_ptr, start_index) as *mut AnyVecPtr::Element;
            let to_drop = ptr::slice_from_raw_parts_mut(start_ptr, end_index - start_index);
            ptr::drop_in_place(to_drop);
        }
    }
}