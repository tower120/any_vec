use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValueCloneable, AnyValueMut, clone_into};
use crate::any_vec_raw::AnyVecRaw;
use crate::any_vec_ptr::{AnyVecPtr, IAnyVecPtr, IAnyVecRawPtr};
use crate::traits::{Cloneable, None, Trait};

// Typed operations will never use type-erased Element, so there is no
// need in type-known-based optimizations.

// TODO: hide. Derive AnyValue staff in Element.
//
/// Owning pointer to [`AnyVec`] element.
///
/// This is public, just so you can see what [`Element`] can do.
///
/// # Notes
///
/// `Element` have it's own implementation of `downcast_` family (which return `&'a T`, instead of `&T`).
/// This is done, so you don't have to keep ElementRef/Mut alive, while casting to concrete type.
/// [`AnyValueMut`] implemented too - for the sake of interface compatibility.
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::get`]: crate::AnyVec::get
pub struct ElementPointer<'a, AnyVecPtr: IAnyVecRawPtr>{
    any_vec_ptr: AnyVecPtr,
    element: NonNull<u8>,
    phantom: PhantomData<&'a mut AnyVecRaw>
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> ElementPointer<'a, AnyVecPtr>{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, element: NonNull<u8>) -> Self {
        Self{any_vec_ptr, element, phantom: PhantomData}
    }

    /// ElementPointer should not be `Clone`, because it can be both & and &mut.
    #[inline]
    pub(crate) fn clone(&self) -> Self {
        Self::new(self.any_vec_ptr, self.element)
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

impl<'a, AnyVecPtr: IAnyVecRawPtr> Drop for ElementPointer<'a, AnyVecPtr>{
    #[inline]
    fn drop(&mut self) {
        if let Some(drop_fn) = self.any_vec_raw().drop_fn(){
            (drop_fn)(self.element.as_ptr(), 1);
        }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr> AnyValue for ElementPointer<'a, AnyVecPtr>{
    type Type = AnyVecPtr::Element;

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

impl<'a, AnyVecPtr: IAnyVecRawPtr> AnyValueMut for ElementPointer<'a, AnyVecPtr>{}

impl<'a, Traits: ?Sized + Cloneable + Trait>
    AnyValueCloneable for ElementPointer<'a, AnyVecPtr<Traits>>
{
    #[inline]
    unsafe fn clone_into(&self, out: *mut u8) {
        clone_into(self, out, self.any_vec_ptr.any_vec().as_ref().clone_fn());
    }
}

unsafe impl<'a, Traits: ?Sized + Send + Trait> Send for ElementPointer<'a, AnyVecPtr<Traits>>{}
unsafe impl<'a, Traits: ?Sized + Sync + Trait> Sync for ElementPointer<'a, AnyVecPtr<Traits>>{}


/// [`AnyVec`] element.
///
/// [`AnyVec`]: crate::AnyVec
pub type Element<'a, Traits> = ElementPointer<'a, AnyVecPtr<Traits>>;


/// Reference to [`Element`].
///
/// Created by  [`AnyVec::get`].
///
/// [`AnyVec::get`]: crate::AnyVec::get
pub struct ElementRef<'a, Traits: ?Sized + Trait = dyn None>(
    pub(crate) ManuallyDrop<ElementPointer<'a, AnyVecPtr<Traits>>>
);
impl<'a, Traits: ?Sized + Trait> Deref for ElementRef<'a, Traits>{
    type Target = Element<'a, Traits>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, Traits: ?Sized + Trait> Clone for ElementRef<'a, Traits>{
    #[inline]
    fn clone(&self) -> Self {
        Self(ManuallyDrop::new(self.0.clone()))
    }
}

/// Mutable reference to [`Element`].
///
/// Created by  [`AnyVec::get_mut`].
///
/// [`AnyVec::get_mut`]: crate::AnyVec::get_mut
pub struct ElementMut<'a, Traits: ?Sized + Trait = dyn None>(
    pub(crate) ManuallyDrop<ElementPointer<'a, AnyVecPtr<Traits>>>
);
impl<'a, Traits: ?Sized + Trait> Deref for ElementMut<'a, Traits>{
    type Target = Element<'a, Traits>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, Traits: ?Sized + Trait> DerefMut for ElementMut<'a, Traits>{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}