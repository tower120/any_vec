use std::iter::FusedIterator;
use crate::Iter;
use crate::traits::Trait;

pub trait Operation{
    type Iter: Iterator;
    fn iter(&self) -> &Self::Iter;
    fn iter_mut(&mut self) -> &mut Self::Iter;
}

/// Iterator over [`AnyVec`] slice. Will do some action on destruction.
pub struct ElementIter<Op: Operation>(pub(crate) Op);

impl<Op: Operation> Iterator
    for ElementIter<Op>
{
    type Item = <Op::Iter as Iterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.iter().size_hint()
    }
}

impl<Op: Operation> DoubleEndedIterator
    for ElementIter<Op>
where
    Op::Iter: DoubleEndedIterator
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().next_back()
    }
}

impl<Op: Operation> ExactSizeIterator
for
    ElementIter<Op>
where
    Op::Iter: ExactSizeIterator
{
    #[inline]
    fn len(&self) -> usize {
        self.0.iter().len()
    }
}

impl<Op: Operation> FusedIterator
for
    ElementIter<Op>
where
    Op::Iter: FusedIterator
{}