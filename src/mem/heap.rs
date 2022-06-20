use std::alloc::{alloc, dealloc, handle_alloc_error, Layout, realloc};
use std::ptr::NonNull;
use crate::mem::{Mem, MemBuilder};


#[derive(Default, Clone)]
pub struct HeapBuilder;
impl MemBuilder<Heap> for HeapBuilder{
    #[inline]
    fn build(&mut self, element_layout: Layout, size: usize) -> Heap {
        let mut this = Heap{
            mem: NonNull::<u8>::dangling(),
            size: 0,
            element_layout
        };
        this.resize(size);
        this
    }
}

pub struct Heap{
    mem: NonNull<u8>,
    size: usize,        // in elements
    element_layout: Layout, // size is aligned
}

impl Mem for Heap{
    type Builder = HeapBuilder;

    #[inline]
    fn as_ptr(&self) -> *const u8 {
        self.mem.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.mem.as_ptr()
    }

    #[inline]
    fn element_layout(&self) -> Layout {
        self.element_layout
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    fn resize(&mut self, new_size: usize) {
        if self.size == new_size{
            return;
        }

        if self.element_layout.size() != 0 {
            unsafe{
                // Non checked mul, because this memory size already allocated.
                let mem_layout = Layout::from_size_align_unchecked(
                    self.element_layout.size() * self.size,
                    self.element_layout.align()
                );

                self.mem =
                    if new_size == 0 {
                        dealloc(self.mem.as_ptr(), mem_layout);
                        NonNull::<u8>::dangling()
                    } else {
                        // mul carefully, to prevent overflow.
                        let new_mem_size = self.element_layout.size()
                            .checked_mul(new_size).unwrap();
                        let new_mem_layout = Layout::from_size_align_unchecked(
                            new_mem_size, self.element_layout.align()
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
        }
        self.size = new_size;
    }
}

impl Drop for Heap{
    fn drop(&mut self) {
        self.resize(0);
    }
}