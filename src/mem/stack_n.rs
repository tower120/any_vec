use std::alloc::Layout;
use std::mem::MaybeUninit;
use crate::mem::{Mem, MemBuilder};

/// Fixed `SIZE` capacity on-stack memory for `N` elements.
///
/// Can contain `N` elements, with total size at most `SIZE` bytes.
/// Unlike [`Stack`] does not involve heavy operations for building.
///
/// [`Stack`]: super::Stack
#[derive(Default, Clone)]
pub struct StackN<const N:usize, const SIZE: usize>;
impl<const N:usize, const SIZE: usize> MemBuilder for StackN<N, SIZE>{
    type Mem = StackNMem<N, SIZE>;

    #[inline]
    fn build(&mut self, element_layout: Layout) -> Self::Mem {
        assert!(N*element_layout.size() <= SIZE, "Insufficient storage!");
        StackNMem{
            mem: MaybeUninit::uninit()
        }
    }

    unsafe fn destroy(&mut self, _: Layout, _: Self::Mem) {}
}

pub struct StackNMem<const N:usize, const SIZE: usize>{
    mem: MaybeUninit<[u8; SIZE]>
}

impl<const N:usize, const SIZE: usize> Mem for StackNMem<N, SIZE>{
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.mem.as_ptr() as *const u8
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.mem.as_mut_ptr() as *mut u8
    }

    #[inline]
    fn size(&self) -> usize {
        N
    }
}