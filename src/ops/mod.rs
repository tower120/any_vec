mod temp;
mod iter;
pub(crate) mod swap_remove;
pub(crate) mod remove;
pub(crate) mod drain;
pub(crate) mod splice;

pub use temp::TempValue;
pub use iter::Iter;

use crate::any_vec_ptr::AnyVecPtr;

/// Lazily `remove` element on consumption/drop.
///
/// This is created by [`AnyVec::remove`].
///
/// [`AnyVec::remove`]: crate::AnyVec::remove
pub type Remove<'a, Traits> = TempValue<remove::Remove<'a, AnyVecPtr<Traits>>, Traits>;

/// Lazily `swap_remove` element on consumption/drop.
///
/// This is created by [`AnyVec::swap_remove`].
///
/// [`AnyVec::swap_remove`]: crate::AnyVec::swap_remove
pub type SwapRemove<'a, Traits> = TempValue<swap_remove::SwapRemove<'a, AnyVecPtr<Traits>>, Traits>;

///  A draining [`ElementIterator`] for [`AnyVec`]. Return [`Element`] items.
///
/// This is created by [`AnyVec::drain`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::drain`]: crate::AnyVec::drain
/// [`Element`]: crate::element::Element
/// [`ElementIterator`]: crate::iter::ElementIterator
pub type Drain<'a, Traits> = Iter<drain::Drain<'a, AnyVecPtr<Traits>>>;

///  A splicing [`ElementIterator`] for [`AnyVec`]. Return [`Element`] items.
///
/// This is created by [`AnyVec::splice`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::splice`]: crate::AnyVec::splice
/// [`Element`]: crate::element::Element
/// [`ElementIterator`]: crate::iter::ElementIterator
pub type Splice<'a, Traits, I> = Iter<splice::Splice<'a, AnyVecPtr<Traits>, I>>;