use std::ops::{Deref, DerefMut};

/// Reference into 'T'.
pub struct Ref<T>(
    pub(crate) T
);
impl<T> Deref for Ref<T>{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Mutable reference into 'T'.
pub struct Mut<T>(
    pub(crate) T
);
impl<T> Deref for Mut<T>{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Mut<T>{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}