use std::iter::{FusedIterator};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use crate::any_value::Unknown;
use crate::any_vec_ptr::{AnyVecPtr, IAnyVecRawPtr};
use crate::any_vec_ptr::utils::element_ptr_at;
use crate::any_vec_raw::AnyVecRaw;
use crate::element::{Element};
use crate::refs::{Mut, Ref};
use crate::traits::Trait;

// TODO :Additional [`AnyVec`] Iterator operations.
/*pub trait AnyVecIterator: Iterator{
    fn lazy_cloned(self) -> impl
}*/

/// [`AnyVec`] iterator.
///
/// [`AnyVec`]: crate::AnyVec
pub struct Iter<'a,
    AnyVecPtr: IAnyVecRawPtr,
    IterItem: IteratorItem<'a, AnyVecPtr> = ElementIterItem<'a, AnyVecPtr>>
{
    pub(crate) any_vec_ptr: AnyVecPtr,

    // TODO: try pointers, instead
    pub(crate) index: usize,
    pub(crate) end: usize,

    phantom: PhantomData<(&'a AnyVecRaw, IterItem)>
}

pub trait IteratorItem<'a, AnyVecPtr: IAnyVecRawPtr>{
    type Item;
    fn element_to_item(element: Element<'a, AnyVecPtr>) -> Self::Item;
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, IterItem: IteratorItem<'a, AnyVecPtr>>
    Iter<'a, AnyVecPtr, IterItem>
{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, start: usize, end: usize) -> Self {
        Self{any_vec_ptr, index: start, end, phantom: PhantomData}
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, IterItem: IteratorItem<'a, AnyVecPtr>> Iterator
    for Iter<'a, AnyVecPtr, IterItem>
{
    type Item = IterItem::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end{
            None
        } else {
            let element_ptr = element_ptr_at(self.any_vec_ptr, self.index);
            let element = Element::new(
                self.any_vec_ptr,
                unsafe{NonNull::new_unchecked(element_ptr)}
            );

            self.index += 1;
            Some(IterItem::element_to_item(element))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.end - self.index;
        (size, Some(size))
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, IterItem: IteratorItem<'a, AnyVecPtr>> DoubleEndedIterator
    for Iter<'a, AnyVecPtr, IterItem>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.index{
            None
        } else {
            self.end -= 1;
            let element_ptr = element_ptr_at(self.any_vec_ptr, self.end);
            let element = Element::new(
                self.any_vec_ptr,
                unsafe{NonNull::new_unchecked(element_ptr)}
            );

            Some(IterItem::element_to_item(element))
        }
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, IterItem: IteratorItem<'a, AnyVecPtr>> ExactSizeIterator
    for Iter<'a, AnyVecPtr, IterItem>
{
    #[inline]
    fn len(&self) -> usize {
        self.end - self.index
    }
}

impl<'a, AnyVecPtr: IAnyVecRawPtr, IterItem: IteratorItem<'a, AnyVecPtr>> FusedIterator
    for Iter<'a, AnyVecPtr, IterItem>
{}

// According to https://github.com/rust-lang/rust/issues/93367#issuecomment-1154832012
#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, Traits: ?Sized + Send + Trait, IterItem: IteratorItem<'a, AnyVecPtr<Traits>>> Send
    for Iter<'a, AnyVecPtr<Traits>, IterItem> {}

#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, Traits: ?Sized + Sync + Trait, IterItem: IteratorItem<'a, AnyVecPtr<Traits>>> Sync
    for Iter<'a, AnyVecPtr<Traits>, IterItem> {}


pub struct ElementIterItem<'a, AnyVecPtr: IAnyVecRawPtr>(
    pub(crate) PhantomData<Element<'a, AnyVecPtr>>
);
impl<'a, AnyVecPtr: IAnyVecRawPtr> IteratorItem<'a, AnyVecPtr> for ElementIterItem<'a, AnyVecPtr>{
    type Item = Element<'a, AnyVecPtr>;

    #[inline]
    fn element_to_item(element: Element<'a, AnyVecPtr>) -> Self::Item {
        element
    }
}


pub struct ElementRefIterItem<'a, AnyVecPtr: IAnyVecRawPtr>(
    pub(crate) PhantomData<Element<'a, AnyVecPtr>>
);
impl<'a, AnyVecPtr: IAnyVecRawPtr> IteratorItem<'a, AnyVecPtr> for ElementRefIterItem<'a, AnyVecPtr>{
    type Item = Ref<ManuallyDrop<Element<'a, AnyVecPtr>>>;

    #[inline]
    fn element_to_item(element: Element<'a, AnyVecPtr>) -> Self::Item {
        Ref(ManuallyDrop::new(element))
    }
}


pub struct ElementMutIterItem<'a, AnyVecPtr: IAnyVecRawPtr>(
    pub(crate) PhantomData<Element<'a, AnyVecPtr>>
);
impl<'a, AnyVecPtr: IAnyVecRawPtr> IteratorItem<'a, AnyVecPtr> for ElementMutIterItem<'a, AnyVecPtr>{
    type Item = Mut<ManuallyDrop<Element<'a, AnyVecPtr>>>;

    #[inline]
    fn element_to_item(element: Element<'a, AnyVecPtr>) -> Self::Item {
        Mut(ManuallyDrop::new(element))
    }
}

//pub type Iter<'a, Traits>    = IterBase<'a, Traits, ElementIterItem<'a, Traits>>;

/// Mutable reference [`AnyVec`] iterator.
///
/// [`AnyVec`]: crate::AnyVec
pub type IterRef<'a, Traits> = Iter<'a, AnyVecPtr<Traits>, ElementRefIterItem<'a, AnyVecPtr<Traits>>>;

/// Reference [`AnyVec`] iterator.
///
/// [`AnyVec`]: crate::AnyVec
pub type IterMut<'a, Traits> = Iter<'a, AnyVecPtr<Traits>, ElementMutIterItem<'a, AnyVecPtr<Traits>>>;