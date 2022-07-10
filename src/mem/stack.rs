use std::alloc::Layout;
use std::mem::MaybeUninit;
use crate::mem::{Mem, MemBuilder};

/// Fixed capacity on-stack memory.
///
/// `SIZE` in bytes.
#[derive(Default, Clone)]
pub struct Stack<const SIZE: usize>;
impl<const SIZE: usize> MemBuilder for Stack<SIZE>{
    type Mem = StackMem<SIZE>;

    #[inline]
    fn build(&mut self, element_layout: Layout) -> StackMem<SIZE> {
        let size =
            if element_layout.size() == 0{
                usize::MAX
            } else{
                SIZE / element_layout.size()
            };

        StackMem{
            mem: MaybeUninit::uninit(),
            element_layout,
            size
        }
    }
}

pub struct StackMem<const SIZE: usize>{
    mem: MaybeUninit<[u8; SIZE]>,
    element_layout: Layout,
    size: usize
}

impl<const SIZE: usize> Mem for StackMem<SIZE>{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.mem.as_ptr() as *const u8
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.mem.as_mut_ptr() as *mut u8
    }

    #[inline]
    fn element_layout(&self) -> Layout {
        self.element_layout
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }
}