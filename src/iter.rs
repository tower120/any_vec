use std::iter::{FusedIterator};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use crate::any_vec_ptr::{AnyVecPtr, IAnyVecRawPtr};
use crate::any_vec_ptr::utils::element_ptr_at;
use crate::any_vec_raw::AnyVecRaw;
use crate::element::{ElementPointer, ElementMut, ElementRef};
use crate::mem::Mem;
use crate::traits::Trait;

// TODO :Additional [`AnyVec`] Iterator operations.
/*pub trait AnyVecIterator: Iterator{
    fn lazy_cloned(self) -> impl
}*/

pub trait ElementIterator:
    DoubleEndedIterator + ExactSizeIterator + FusedIterator
{}

impl<T> ElementIterator for T
where
    T: DoubleEndedIterator + ExactSizeIterator + FusedIterator
{}

/// [`AnyVec`] iterator.
///
/// Return [`Element`], [`ElementRef`] or [`ElementMut`] items, depending on `IterItem`.
/// Cloneable for `Ref` and `Mut` versions.
///
/// [`AnyVec`]: crate::AnyVec
/// [`Element`]: crate::element::Element
/// [`ElementRef`]: crate::element::ElementRef
/// [`ElementMut`]: crate::element::ElementMut
#[derive(Clone)]
pub struct Iter<'a,
    M: Mem,
    AnyVecPtr: IAnyVecRawPtr<M>,
    IterItem: IteratorItem<'a, M, AnyVecPtr> = ElementIterItem<'a, M, AnyVecPtr>>
{
    pub(crate) any_vec_ptr: AnyVecPtr,

    // TODO: try pointers, instead
    pub(crate) index: usize,
    pub(crate) end: usize,

    phantom: PhantomData<(&'a AnyVecRaw<M>, IterItem)>
}

impl<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>, IterItem: IteratorItem<'a, M, AnyVecPtr>>
    Iter<'a, AnyVecPtr, IterItem>
{
    #[inline]
    pub(crate) fn new(any_vec_ptr: AnyVecPtr, start: usize, end: usize) -> Self {
        Self{any_vec_ptr, index: start, end, phantom: PhantomData}
    }
}

impl<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>, IterItem: IteratorItem<'a, M, AnyVecPtr>> Iterator
    for Iter<'a, M, AnyVecPtr, IterItem>
{
    type Item = IterItem::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end{
            None
        } else {
            let element_ptr = element_ptr_at(self.any_vec_ptr, self.index);
            let element = ElementPointer::new(
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

impl<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>, IterItem: IteratorItem<'a, AnyVecPtr>> DoubleEndedIterator
    for Iter<'a, M, AnyVecPtr, IterItem>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.index{
            None
        } else {
            self.end -= 1;
            let element_ptr = element_ptr_at(self.any_vec_ptr, self.end);
            let element = ElementPointer::new(
                self.any_vec_ptr,
                unsafe{NonNull::new_unchecked(element_ptr)}
            );

            Some(IterItem::element_to_item(element))
        }
    }
}

impl<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>, IterItem: IteratorItem<'a, M, AnyVecPtr>> ExactSizeIterator
    for Iter<'a, M, AnyVecPtr, IterItem>
{
    #[inline]
    fn len(&self) -> usize {
        self.end - self.index
    }
}

impl<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>, IterItem: IteratorItem<'a, M, AnyVecPtr>> FusedIterator
    for Iter<'a, M, AnyVecPtr, IterItem>
{}

// According to https://github.com/rust-lang/rust/issues/93367#issuecomment-1154832012
#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, M: Mem, Traits: ?Sized + Send + Trait, IterItem: IteratorItem<'a, M, AnyVecPtr<Traits, M>>> Send
    for Iter<'a, M, AnyVecPtr<Traits, M>, IterItem> {}

#[allow(suspicious_auto_trait_impls)]
unsafe impl<'a, M: Mem, Traits: ?Sized + Sync + Trait, IterItem: IteratorItem<'a, M, AnyVecPtr<Traits, M>>> Sync
    for Iter<'a, M, AnyVecPtr<Traits, M>, IterItem> {}


pub trait IteratorItem<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>>{
    type Item;
    fn element_to_item(element: ElementPointer<'a, M, AnyVecPtr>) -> Self::Item;
}

/// Default
pub struct ElementIterItem<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>>(
    pub(crate) PhantomData<ElementPointer<'a, M, AnyVecPtr>>
);
impl<'a, M: Mem, AnyVecPtr: IAnyVecRawPtr<M>> IteratorItem<'a, M, AnyVecPtr> for ElementIterItem<'a, M, AnyVecPtr>{
    type Item = ElementPointer<'a, M, AnyVecPtr>;

    #[inline]
    fn element_to_item(element: ElementPointer<'a, M, AnyVecPtr>) -> Self::Item {
        element
    }
}

/// Ref
pub struct ElementRefIterItem<'a, Traits: ?Sized + Trait, M: Mem>(
    pub(crate) PhantomData<ElementPointer<'a, M, AnyVecPtr<Traits, M>>>
);
impl<'a, Traits: ?Sized + Trait, M: Mem> IteratorItem<'a, M, AnyVecPtr<Traits, M>> for ElementRefIterItem<'a, Traits, M>{
    type Item = ElementRef<'a, Traits, M>;

    #[inline]
    fn element_to_item(element: ElementPointer<'a, M, AnyVecPtr<Traits, M>>) -> Self::Item {
        ElementRef(ManuallyDrop::new(element))
    }
}
impl<'a, Traits: ?Sized + Trait, M: Mem> Clone for ElementRefIterItem<'a, M, Traits>{
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}


/// Mut
pub struct ElementMutIterItem<'a, Traits: ?Sized + Trait, M: Mem>(
    pub(crate) PhantomData<ElementPointer<'a, M, AnyVecPtr<Traits, M>>>
);
impl<'a, Traits: ?Sized + Trait, M: Mem> IteratorItem<'a, M, AnyVecPtr<Traits, M>> for ElementMutIterItem<'a, Traits, M>{
    type Item = ElementMut<'a, Traits>;

    #[inline]
    fn element_to_item(element: ElementPointer<'a, M, AnyVecPtr<Traits, M>>) -> Self::Item {
        ElementMut(ManuallyDrop::new(element))
    }
}
impl<'a, Traits: ?Sized + Trait, M: Mem> Clone for ElementMutIterItem<'a, Traits, M>{
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}


//pub type Iter<'a, Traits>    = IterBase<'a, Traits, ElementIterItem<'a, Traits>>;

/// Reference [`AnyVec`] iterator. Return [`ElementRef`] items.
///
/// [`AnyVec`]: crate::AnyVec
/// [`ElementRef`]: crate::element::ElementRef
pub type IterRef<'a, Traits, M> = Iter<'a, M, AnyVecPtr<Traits, M>, ElementRefIterItem<'a, Traits, M>>;

/// Mutable reference [`AnyVec`] iterator. Return [`ElementMut`] items.
///
/// [`AnyVec`]: crate::AnyVec
/// [`ElementMut`]: crate::element::ElementMut
pub type IterMut<'a, Traits> = Iter<'a, AnyVecPtr<Traits>, ElementMutIterItem<'a, Traits>>;