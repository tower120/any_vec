use std::alloc::Layout;
use crate::mem::{dangling, Mem, MemBuilder};

/// Zero-size memory.
///
/// Contain only element Layout. This can be useful for constructing [`RawParts`] with zero overhead.
///
/// [`RawParts`]: crate::RawParts
#[derive(Default, Clone, Copy)]
pub struct Empty;
impl MemBuilder for Empty{
    type Mem = EmptyMem;

    #[inline]
    fn build(&mut self, element_layout: Layout) -> Self::Mem {
        EmptyMem { element_layout }
    }
}

pub struct EmptyMem{
    element_layout: Layout
}

impl Mem for EmptyMem{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        dangling(&self.element_layout).as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        dangling(&self.element_layout).as_ptr()
    }

    #[inline]
    fn element_layout(&self) -> Layout {
        self.element_layout
    }

    #[inline]
    fn size(&self) -> usize {
        0
    }
}