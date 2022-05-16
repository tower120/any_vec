use std::ops::{Deref, DerefMut};
use crate::{AnyVec, AnyVecRef, AnyVecTyped};

pub struct AnyVecMut<'a, T>{
    pub(crate) any_vec_typed: AnyVecTyped<'a, T>
}

impl<'a, T> Deref for AnyVecMut<'a, T> {
    type Target = AnyVecTyped<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.any_vec_typed
    }
}

impl<'a, T> DerefMut for AnyVecMut<'a, T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.any_vec_typed
    }
}