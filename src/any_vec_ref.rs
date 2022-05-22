use std::ops::{Deref};
use crate::{AnyVecTyped};

/// Typed view to &[`AnyVec`].
///
/// You can get it from [`AnyVec::downcast_ref`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::downcast_ref`]: crate::AnyVec::downcast_ref
pub struct AnyVecRef<'a, T: 'static>{
    pub(crate) any_vec_typed: AnyVecTyped<'a, T>
}

impl<'a, T: 'static> Deref for AnyVecRef<'a, T> {
    type Target = AnyVecTyped<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.any_vec_typed
    }
}

// Just in case
/*impl<'a, T: 'static> From<AnyVecMut<'a, T>> for AnyVecRef<'a, T> {
    fn from(any_vec_mut: AnyVecMut<'a, T>) -> Self {
        Self{
            any_vec_typed: any_vec_mut.any_vec_typed
        }
    }
}*/