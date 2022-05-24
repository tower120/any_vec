use std::alloc::Layout;
use std::any::TypeId;
use crate::{AnyVecMut, AnyVecRef};
use crate::any_value::AnyValue;
use crate::any_vec_raw::AnyVecRaw;
use crate::ops::{AnyValueTemp, Remove, SwapRemove};

pub struct AnyVec {
    raw: AnyVecRaw
}

impl AnyVec {
    pub fn new<Element: 'static>() -> Self {
        Self::with_capacity::<Element>(0)
    }

    pub fn with_capacity<Element: 'static>(capacity: usize) -> Self {
        Self{
            raw: AnyVecRaw::with_capacity::<Element>(capacity)
        }
    }

    #[inline]
    pub fn downcast_ref<Element: 'static>(&self) -> Option<AnyVecRef<Element>> {
        self.raw.downcast_ref::<Element>()
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<Element: 'static>(&self) -> AnyVecRef<Element> {
        self.raw.downcast_ref_unchecked::<Element>()
    }

    #[inline]
    pub fn downcast_mut<Element: 'static>(&mut self) -> Option<AnyVecMut<Element>> {
        self.raw.downcast_mut::<Element>()
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<Element: 'static>(&mut self) -> AnyVecMut<Element> {
        self.raw.downcast_mut_unchecked::<Element>()
    }

    pub fn insert<V: AnyValue>(&mut self, index: usize, value: V) {
        self.raw.insert(index, value);
    }

    #[inline]
    pub fn push<V: AnyValue>(&mut self, value: V) {
        self.raw.push(value);
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> AnyValueTemp<Remove> {
        self.raw.remove(index)
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> AnyValueTemp<SwapRemove> {
        self.raw.swap_remove(index)
    }

    #[inline]
    pub fn clear(&mut self){
        self.raw.clear()
    }

    /// Element TypeId
    #[inline]
    pub fn element_typeid(&self) -> TypeId{
        self.raw.element_typeid()
    }

    /// Element Layout
    #[inline]
    pub fn element_layout(&self) -> Layout {
        self.raw.element_layout()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }
}