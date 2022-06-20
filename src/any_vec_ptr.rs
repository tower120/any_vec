//! Type dispatched analog of `enum{*AnyVecRaw, *AnyVec<Traits>}`.

use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::any_value::Unknown;
use crate::any_vec_raw::AnyVecRaw;
use crate::AnyVec;
use crate::mem::Mem;
use crate::traits::Trait;

pub trait IAnyVecRawPtr: Copy{
    /// Known element type of AnyVec
    type Element: 'static/* = Unknown*/;
    // TODO: rename to Mem?
    type M: Mem;
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw<Self::M>>;
}
pub trait IAnyVecPtr: IAnyVecRawPtr{
    type Traits: ?Sized + Trait;
    fn any_vec(&self) -> NonNull<AnyVec<Self::Traits, Self::M>>;
}


pub struct AnyVecRawPtr<Type: 'static, M: Mem>{
    ptr: NonNull<AnyVecRaw<M>>,
    phantom: PhantomData<*mut Type>
}
impl<Type, M: Mem> From<NonNull<AnyVecRaw<M>>> for AnyVecRawPtr<Type, M>{
    #[inline]
    fn from(ptr: NonNull<AnyVecRaw<M>>) -> Self {
        Self{ptr, phantom: PhantomData}
    }
}
impl<Type, M: Mem> Copy for AnyVecRawPtr<Type, M> {}
impl<Type, M: Mem> Clone for AnyVecRawPtr<Type, M> {
    #[inline]
    fn clone(&self) -> Self {
        Self{
            ptr: self.ptr,
            phantom: PhantomData
        }
    }
}

impl<Type, M: Mem> IAnyVecRawPtr for AnyVecRawPtr<Type, M>{
    type Element = Type;
    type M = M;

    #[inline]
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw<M>> {
        self.ptr
    }
}


pub struct AnyVecPtr<Traits: ?Sized + Trait, M: Mem>{
    ptr: NonNull<AnyVec<Traits, M>>
}
impl<Traits: ?Sized + Trait, M: Mem> From<NonNull<AnyVec<Traits, M>>> for AnyVecPtr<Traits, M> {
    #[inline]
    fn from(ptr: NonNull<AnyVec<Traits, M>>) -> Self {
        Self{ptr}
    }
}
impl<Traits: ?Sized + Trait, M: Mem> From<&mut AnyVec<Traits, M>> for AnyVecPtr<Traits, M> {
    #[inline]
    fn from(reference: &mut AnyVec<Traits, M>) -> Self {
        Self{ptr: NonNull::from(reference)}
    }
}
impl<Traits: ?Sized + Trait, M: Mem> From<&AnyVec<Traits, M>> for AnyVecPtr<Traits, M> {
    #[inline]
    fn from(reference: &AnyVec<Traits, M>) -> Self {
        Self{ptr: NonNull::from(reference)}
    }
}
impl<Traits: ?Sized + Trait, M: Mem> Clone for AnyVecPtr<Traits, M>{
    #[inline]
    fn clone(&self) -> Self {
        Self{ptr: self.ptr}
    }
}
impl<Traits: ?Sized + Trait, M: Mem> Copy for AnyVecPtr<Traits, M>{}

impl<Traits: ?Sized + Trait, M: Mem> IAnyVecRawPtr for AnyVecPtr<Traits, M> {
    type Element = Unknown;
    type M = M;

    #[inline]
    fn any_vec_raw(&self) -> NonNull<AnyVecRaw<M>> {
        NonNull::from(unsafe{&self.ptr.as_ref().raw})
    }
}
impl<Traits: ?Sized + Trait, M: Mem> IAnyVecPtr for AnyVecPtr<Traits, M> {
    type Traits = Traits;

    #[inline]
    fn any_vec(&self) -> NonNull<AnyVec<Traits, M>> {
        self.ptr
    }
}


/// Type knowledge optimized operations.
pub(crate) mod utils{
    use std::{mem, ptr};
    use std::mem::size_of;
    use crate::mem::Mem;
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
            any_vec_raw.mem.as_mut_ptr()
                .add(any_vec_raw.element_layout().size() * index)
        } else {
            any_vec_raw.mem.as_mut_ptr().cast::<AnyVecPtr::Element>()
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