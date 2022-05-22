use std::ops::{Deref, DerefMut};
use crate::{AnyVecTyped};

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