use std::alloc::{alloc, dealloc, handle_alloc_error, Layout, realloc};
use std::cmp;
use std::ptr::NonNull;
use crate::mem::{Mem, MemBuilder, MemBuilderSizeable, MemResizable};

#[inline]
fn dangling(layout: &Layout) -> NonNull<u8>{
    #[cfg(miri)]
    {
        layout.dangling()
    }
    #[cfg(not(miri))]
    {
        unsafe { NonNull::new_unchecked(layout.align() as *mut u8) }
    }
}

/// Heap allocated memory.
#[derive(Default, Clone)]
pub struct Heap;
impl MemBuilder for Heap {
    type Mem = HeapMem;

    #[inline]
    fn build(&mut self, element_layout: Layout) -> HeapMem {
        HeapMem {
            mem: dangling(&element_layout),
            size: 0
        }
    }

    #[inline]
    unsafe fn destroy(&mut self, element_layout: Layout, mut mem: Self::Mem) {
        mem.resize(element_layout, 0);
    }
}
impl MemBuilderSizeable for Heap{
    #[inline]
    fn build_with_size(&mut self, element_layout: Layout, capacity: usize) -> Self::Mem
    {
        let mut mem = self.build(element_layout);
        unsafe{
            mem.resize(element_layout, capacity);
        }
        mem
    }
}

pub struct HeapMem {
    mem: NonNull<u8>,
    size: usize,        // in elements
}

impl Mem for HeapMem {
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.mem.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.mem.as_ptr()
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    unsafe fn expand(&mut self, element_layout: Layout, additional: usize){
        let requested_size = self.size() + additional;
        let new_size = cmp::max(self.size() * 2, requested_size);
        self.resize(element_layout, new_size);
    }
}

impl MemResizable for HeapMem {
    unsafe fn resize(&mut self, element_layout: Layout, new_size: usize) {
        if self.size == new_size{
            return;
        }

        if element_layout.size() != 0 {
            // Non checked mul, because this memory size already allocated.
            let mem_layout = Layout::from_size_align_unchecked(
                element_layout.size() * self.size,
                element_layout.align()
            );

            self.mem =
                if new_size == 0 {
                    dealloc(self.mem.as_ptr(), mem_layout);
                    dangling(&element_layout)
                } else {
                    // mul carefully, to prevent overflow.
                    let new_mem_size = element_layout.size()
                        .checked_mul(new_size).unwrap();
                    let new_mem_layout = Layout::from_size_align_unchecked(
                        new_mem_size, element_layout.align()
                    );

                    if self.size == 0 {
                        // allocate
                        NonNull::new(alloc(new_mem_layout))
                    } else {
                        // reallocate
                        NonNull::new(realloc(
                            self.mem.as_ptr(), mem_layout,new_mem_size
                        ))
                    }
                    .unwrap_or_else(|| handle_alloc_error(new_mem_layout))
                }
        }
        self.size = new_size;
    }
}

unsafe impl Send for HeapMem{}
unsafe impl Sync for HeapMem{}