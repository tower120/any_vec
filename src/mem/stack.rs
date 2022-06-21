use std::alloc::Layout;
use std::mem::MaybeUninit;
use crate::mem::{Mem, MemBuilder};

#[derive(Default, Clone)]
pub struct Stack<const SIZE: usize>;
impl<const SIZE: usize> MemBuilder for Stack<SIZE>{
    type Mem = StackMem<SIZE>;

    #[inline]
    fn build(&mut self, element_layout: Layout) -> StackMem<SIZE> {
        StackMem{
            mem: MaybeUninit::uninit(),
            element_layout
        }
    }
}

/// Fixed capacity on-stack memory.
///
/// `SIZE` in bytes.
pub struct StackMem<const SIZE: usize>{
    mem: MaybeUninit<[u8; SIZE]>,
    element_layout: Layout
}

impl<const SIZE: usize> Mem for StackMem<SIZE>{
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