mod temp;
pub(crate) mod swap_remove;
pub(crate) mod remove;
pub(crate) mod drain;
mod drain_filter;

pub use temp::TempValue;

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

/// Same as [`std::vec::Drain`].
///
/// This is created by [`AnyVec::drain`].
///
/// [`AnyVec::drain`]: crate::AnyVec::drain
pub type Drain<'a, Traits> = drain::Drain<'a, AnyVecPtr<Traits>>;