use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValueCloneable, AnyValueMut, clone_into, Unknown};
use crate::{refs};
use crate::any_vec_raw::AnyVecRaw;
use crate::any_vec_ptr::{AnyVecPtr, IAnyVecPtr, IAnyVecRawPtr};
use crate::traits::{Cloneable, Trait};

// Typed operations will never use type-erased Element, so there is no
// need in type-known-based optimizations.

/// Owning pointer to [`AnyVec`] element.
///
/// Crated with [`AnyVec::get`] -family.
/// Accessed through [`ElementRef`] or [`ElementMut`].
///
/// # Notes
///
/// `Element` have it's own implementation of `downcast_` family (which return `&'a T`, instead of `&T`).
/// This is done, so you don't have to keep ElementRef/Mut alive, while casting to concrete type.
/// [`AnyValueMut`] implemented too - for the sake of interface compatibility.
pub struct Element<'a, AnyVecPtr: IAnyVecRawPtr>{
    any_vec_ptr: AnyVecPtr,
    element: NonNull<u8>,
    phantom: PhantomData<&'a mut AnyVecRaw>
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> Element<'a, AnyVecPtr>{
    #[inline]
    pub fn new(any_vec_ptr: AnyVecPtr, element: NonNull<u8>) -> Self {
        Self{any_vec_ptr, element, phantom: PhantomData}
    }

    #[inline]
    fn any_vec_raw(&self) -> &'a AnyVecRaw{
        unsafe { self.any_vec_ptr.any_vec_raw().as_ref() }
    }

    #[inline]
    pub fn downcast_ref<T: 'static>(&self) -> Option<&'a T>{
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_ref_unchecked::<T>() })
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &'a T{
        &*(self.bytes() as *const T)
    }

    #[inline]
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&'a mut T>{
        if self.value_typeid() != TypeId::of::<T>(){
            None
        } else {
            Some(unsafe{ self.downcast_mut_unchecked::<T>() })
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &'a mut T{
        &mut *(self.bytes_mut() as *mut T)
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> Drop for Element<'a, AnyVecPtr>{
    #[inline]
    fn drop(&mut self) {
        if let Some(drop_fn) = self.any_vec_raw().drop_fn(){
            (drop_fn)(self.element.as_ptr(), 1);
        }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> AnyValue for Element<'a, AnyVecPtr>{
    type Type = Unknown;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.any_vec_raw().element_typeid()
    }

    #[inline]
    fn size(&self) -> usize {
        self.any_vec_raw().element_layout().size()
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.element.as_ptr()
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> AnyValueMut for Element<'a, AnyVecPtr>{}

impl<'a, Traits: ?Sized + Cloneable + Trait>
    AnyValueCloneable for Element<'a, AnyVecPtr<Traits>>
{
    #[inline]
    unsafe fn clone_into(&self, out: *mut u8) {
        clone_into(self, out, self.any_vec_ptr.any_vec().as_ref().clone_fn());
    }
}

unsafe impl<'a, Traits: ?Sized + Send + Trait> Send for Element<'a, AnyVecPtr<Traits>>{}
unsafe impl<'a, Traits: ?Sized + Sync + Trait> Sync for Element<'a, AnyVecPtr<Traits>>{}

/// Reference to [`Element`].
///
/// Created by  [`AnyVec::get`].
pub type ElementRef<'a, Traits> = refs::Ref<ManuallyDrop<
    Element<'a, AnyVecPtr<Traits>>
>>;

/// Mutable reference to [`Element`].
///
/// Created by  [`AnyVec::get_mut`].
pub type ElementMut<'a, Traits> = refs::Mut<ManuallyDrop<
    Element<'a, AnyVecPtr<Traits>>
>>;