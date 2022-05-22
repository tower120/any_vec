use std::ops::{Deref, DerefMut};
use crate::{AnyVecTyped};

/// Typed view to &mut [`AnyVec`].
///
/// You can get it from [`AnyVec::downcast_mut`].
///
/// [`AnyVec`]: crate::AnyVec
/// [`AnyVec::downcast_mut`]: crate::AnyVec::downcast_mut
pub struct AnyVecMut<'a, T: 'static>{
    pub(crate) any_vec_typed: AnyVecTyped<'a, T>
}

impl<'a, T: 'static> Deref for AnyVecMut<'a, T> {
    type Target = AnyVecTyped<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.any_vec_typed
    }
}

impl<'a, T: 'static> DerefMut for AnyVecMut<'a, T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.any_vec_typed
    }
}