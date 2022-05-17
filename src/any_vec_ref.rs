use std::ops::{Deref};
use crate::{AnyVecTyped};

pub struct AnyVecRef<'a, T>{
    pub(crate) any_vec_typed: AnyVecTyped<'a, T>
}

impl<'a, T> Deref for AnyVecRef<'a, T> {
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