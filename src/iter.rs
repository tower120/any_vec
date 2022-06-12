use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;
use crate::any_vec_ptr::AnyVecPtr;
use crate::any_vec_raw::AnyVecRaw;
use crate::AnyVec;
use crate::element::{Element, ElementMut, ElementRef};
use crate::refs::{Mut, Ref};
use crate::traits::Trait;

// TODO :Additional [`AnyVec`] Iterator operations.
/*pub trait AnyVecIterator: Iterator{
    fn lazy_cloned(self) -> impl
}*/

/// [`AnyVec`] iterator
pub struct Iter<'a,
    Traits: ?Sized + Trait,
    IterItem: IteratorItem<'a, Traits> = ElementIterItem<'a, Traits>>
{
    pub(crate) any_vec: NonNull<AnyVec<Traits>>,

    // TODO: try pointers, instead
    pub(crate) start: usize,
    pub(crate) end: usize,

    // TODO: try just &'a AnyVec<Traits>
    phantom: PhantomData<(&'a AnyVecRaw, IterItem)>
}

pub trait IteratorItem<'a, Traits: ?Sized + Trait>{
    type Item;
    fn element_to_item(element: Element<'a, AnyVecPtr<Traits>>) -> Self::Item;
}

impl<'a, Traits: ?Sized + Trait, IterItem: IteratorItem<'a, Traits>>
    Iter<'a, Traits, IterItem>
{
    #[inline]
    pub(crate) fn new(any_vec: NonNull<AnyVec<Traits>>, start: usize, end: usize) -> Self {
        Self{any_vec, start, end, phantom: PhantomData}
    }
}

impl<'a, Traits: ?Sized + Trait, IterItem: IteratorItem<'a, Traits>> Iterator
    for Iter<'a, Traits, IterItem>
{
    type Item = IterItem::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end{
            None
        } else {
            unsafe{
                let any_vec_raw = &self.any_vec.as_ref().raw;
                let size = any_vec_raw.element_layout().size();
                let element_ptr = any_vec_raw.mem.as_ptr().add(size * self.start);
                let element = Element::new(
                    AnyVecPtr::from(self.any_vec),
                    NonNull::new_unchecked(element_ptr)
                );

                self.start += 1;
                Some(IterItem::element_to_item(element))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.end - self.start;
        (size, Some(size))
    }
}



pub struct ElementIterItem<'a, Traits: ?Sized + Trait>(
    pub(crate) PhantomData<Element<'a, AnyVecPtr<Traits>>>
);
impl<'a, Traits: ?Sized + Trait> IteratorItem<'a, Traits> for ElementIterItem<'a, Traits>{
    type Item = Element<'a, AnyVecPtr<Traits>>;

    #[inline]
    fn element_to_item(element: Element<'a, AnyVecPtr<Traits>>) -> Self::Item {
        element
    }
}

pub struct ElementRefIterItem<'a, Traits: ?Sized + Trait>(
    pub(crate) PhantomData<ElementRef<'a, Traits>>
);
impl<'a, Traits: ?Sized + Trait> IteratorItem<'a, Traits> for ElementRefIterItem<'a, Traits>{
    type Item = ElementRef<'a, Traits>;

    #[inline]
    fn element_to_item(element: Element<'a, AnyVecPtr<Traits>>) -> Self::Item {
        Ref(ManuallyDrop::new(element))
    }
}


pub struct ElementMutIterItem<'a, Traits: ?Sized + Trait>(
    pub(crate) PhantomData<ElementMut<'a, Traits>>
);
impl<'a, Traits: ?Sized + Trait> IteratorItem<'a, Traits> for ElementMutIterItem<'a, Traits>{
    type Item = ElementMut<'a, Traits>;

    #[inline]
    fn element_to_item(element: Element<'a, AnyVecPtr<Traits>>) -> Self::Item {
        Mut(ManuallyDrop::new(element))
    }
}

//pub type Iter<'a, Traits>    = IterBase<'a, Traits, ElementIterItem<'a, Traits>>;
pub type IterRef<'a, Traits> = Iter<'a, Traits, ElementRefIterItem<'a, Traits>>;
pub type IterMut<'a, Traits> = Iter<'a, Traits, ElementMutIterItem<'a, Traits>>;