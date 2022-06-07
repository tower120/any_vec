use std::any::TypeId;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValueCloneable, AnyValueMut, clone_into, LazyClone, Unknown};
use crate::{AnyVec, refs};
use crate::traits::{Cloneable, Trait};

/// Pointer to [`AnyVec`] element.
///
/// Crated with [`AnyVec::get`] -family.
/// Accessed through [`ElementRef`] or [`ElementMut`].
///
/// # Notes
///
/// `Element` reimplement [`AnyValueMut`] `downcast_` family, in order to return `&'a T`,
/// instead of `&T`.Without that, you would have to keep ElementRef alive,
/// while casting to concrete type.
pub struct Element<'a, Traits: ?Sized + Trait>{
    pub(crate) any_vec: &'a AnyVec<Traits>,
    pub(crate) element: *const u8
}

impl<'a, Traits: ?Sized + Trait> Element<'a, Traits>{
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

impl<'a, Traits: ?Sized + Trait> AnyValue for Element<'a, Traits>{
    type Type = Unknown;

    fn value_typeid(&self) -> TypeId {
        self.any_vec.element_typeid()
    }

    fn size(&self) -> usize {
        self.any_vec.element_layout().size()
    }

    fn bytes(&self) -> *const u8 {
        self.element
    }
}

impl<'a, Traits: ?Sized + Trait> AnyValueMut for Element<'a, Traits>{}

impl<'a, Traits: ?Sized + Cloneable + Trait> AnyValueCloneable for Element<'a, Traits>{
    unsafe fn clone_into(&self, out: *mut u8) {
        clone_into(self, out, self.any_vec.clone_fn());
    }
}

unsafe impl<'a, Traits: ?Sized + Send + Trait> Send for Element<'a, Traits>{}
unsafe impl<'a, Traits: ?Sized + Sync + Trait> Sync for Element<'a, Traits>{}

/// Reference to ['AnyVec'] element.
///
/// Created by  ['AnyVec::get'].
pub type ElementRef<'a, Traits> = refs::Ref<Element<'a, Traits>>;

/// Mutable reference to ['AnyVec'] element.
///
/// Created by  ['AnyVec::get_mut'].
pub type ElementMut<'a, Traits> = refs::Mut<Element<'a, Traits>>;