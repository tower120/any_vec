use std::alloc::Layout;
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::{NonNull, null_mut};
use crate::any_value::{AnyValue, AnyValueRaw, Unknown};
use crate::any_vec_raw::AnyVecRaw;
use crate::AnyVec;
use crate::clone_type::CloneType;
use crate::traits::{Cloneable, Trait};


pub struct ElementMut<'a, Traits: ?Sized + Trait>(Element<'a, Traits>);
impl<'a, Traits: ?Sized + Trait> ElementMut<'a, Traits>{
    /// Unsafe, because Element origin is unchecked
    #[inline]
    pub(crate) unsafe fn new(element: Element<'a, Traits>) -> Self{
        Self(element)
    }

    #[inline]
    pub fn downcast_ref<T: 'static>(&self) -> Option<&'a T>{
    unsafe{
        if self.0.any_vec.as_ref().element_typeid() == TypeId::of::<T>() {
            Some( self.downcast_ref_unchecked() )
        } else {
            None
        }
    }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &'a T{
        self.0.any_vec.as_ref().downcast_ref_unchecked::<T>().as_slice().get_unchecked(self.0.index)
    }

    #[inline]
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&'a T>{
    unsafe{
        if self.0.any_vec.as_ref().element_typeid() == TypeId::of::<T>() {
            Some( self.downcast_mut_unchecked() )
        } else {
            None
        }
    }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &'a mut T{
        self.0.any_vec.as_mut().downcast_mut_unchecked::<T>().as_mut_slice().get_unchecked_mut(self.0.index)
    }
}
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


// TODO: Clone ?
pub struct ElementRef<'a, Traits: ?Sized + Trait>(ElementMut<'a, Traits>);
impl<'a, Traits: ?Sized + Trait> ElementRef<'a, Traits>{
    /// Unsafe, because Element origin is unchecked
    #[inline]
    pub(crate) unsafe fn new(element: Element<'a, Traits>) -> Self{
        Self(ElementMut::new(element))
    }

    #[inline]
    pub fn downcast_ref<T: 'static>(&self) -> Option<&'a T>{
        self.0.downcast_ref::<T>()
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &'a T{
        self.0.downcast_ref_unchecked::<T>()
    }
}
impl<'a, Traits: ?Sized + Trait> Deref for ElementRef<'a, Traits>{
    type Target = Element<'a, Traits>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
}



/// If `Traits` implement Cloneable - `Element` implements [`Clone`] and [`AnyValue`].
///
/// `Element` lazily clone on consumption as [`AnyValue`].
///
/// # Example
// TODO: Example
// TODO: rename to LazyClone
pub struct Element<'a, Traits: ?Sized + Trait>{
    // NonNull instead of &'a because it is used by both ElementRef and ElementMut
    pub(crate) any_vec: NonNull<AnyVec<Traits>>,
    pub(crate) index: usize,
    pub(crate) phantom: PhantomData<&'a mut AnyVec<Traits>>
}

impl<'a, Traits: ?Sized + Trait> Element<'a, Traits> {
    #[inline]
    fn any_vec(&self) -> &'a AnyVec<Traits>{
        unsafe{ self.any_vec.as_ref() }
    }
}

/// Lazy clone on consumption
impl<'a, Traits: ?Sized + Cloneable + Trait> AnyValue for Element<'a, Traits>{
    type Type = Unknown;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        self.any_vec().element_typeid()
    }

    #[inline]
    fn value_size(&self) -> usize {
        self.any_vec().element_layout().size()
    }

    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F) {
        const MAX_STACK_SIZE: usize = 512;

        // allocate
        let layout = self.any_vec().element_layout();
        let mut tmp_stack: MaybeUninit<[u8; MAX_STACK_SIZE]> = MaybeUninit::uninit();
        let tmp_ptr =
            if layout.size() > MAX_STACK_SIZE {
                std::alloc::alloc(layout)
            } else {
                tmp_stack.as_mut_ptr() as *mut u8
            };

        // consume
        self.consume_bytes_into(tmp_ptr);
        f(NonNull::new_unchecked(tmp_ptr));

        // deallocate
        if layout.size() > MAX_STACK_SIZE {
            std::alloc::dealloc(tmp_ptr, layout);
        }
    }

    unsafe fn consume_bytes_into(self, out: *mut u8) {
        let ptr = self.any_vec().as_bytes().add(self.value_size() * self.index);

        // clone out
        if let Some(clone_fn)= <Traits as CloneType>::get(self.any_vec().clone_fn){
            (clone_fn)(ptr, out, 1);
        } else {
            ptr::copy_nonoverlapping(ptr, out, self.value_size());
        }
    }
}

impl<'a, Traits: ?Sized + Cloneable + Trait> Clone for Element<'a, Traits>{
    #[inline]
    fn clone(&self) -> Self {
        Self{
            any_vec: self.any_vec,
            index: self.index,
            phantom: PhantomData
        }
    }
}