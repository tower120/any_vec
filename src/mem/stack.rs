use std::alloc::Layout;
use std::mem::MaybeUninit;
use crate::mem::{Mem, MemBuilder};

/// Fixed capacity on-stack memory.
///
/// `SIZE` in bytes.
pub struct Stack<const SIZE: usize>{
    mem: MaybeUninit<[u8; SIZE]>,
    element_layout: Layout
}

#[derive(Default, Clone)]
pub struct StackBuilder;
impl<const SIZE: usize> MemBuilder<Stack<SIZE>> for StackBuilder{
    #[inline]
    fn build(&mut self, element_layout: Layout, size: usize) -> Stack<SIZE> {
        assert!(size <= SIZE, "Requested mem size too big!");
        Stack{
            mem: MaybeUninit::uninit(),
            element_layout
        }
    }
}

impl<const SIZE: usize> Mem for Stack<SIZE>{
    type Builder = StackBuilder;

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        unsafe{self.mem.assume_init_ref()}.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe{self.mem.assume_init_mut()}.as_mut_ptr()
    }

    #[inline]
    fn element_layout(&self) -> Layout {
        self.element_layout
    }

    #[inline]
    fn size(&self) -> usize {
        SIZE
    }
}