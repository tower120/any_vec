mod heap;

pub use heap::Heap;

pub(crate) type Default = Heap;

use std::alloc::Layout;
use std::cmp;
use std::ptr::NonNull;

/// Memory interface for [`AnyVec`].
///
/// Responsible for allocation and dealocation of the memory chunk.
/// Memory chunk is always one.
pub trait Mem{
    fn new(element_layout: Layout, capacity: usize) -> Self;

    fn as_ptr(&self) -> *const u8;

    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Aligned.
    fn element_layout(&self) -> Layout;

    /// In elements.
    fn size(&self) -> usize;

    fn expand(&mut self, additional: usize){
        let requested_size = self.size() + additional;
        let new_size = cmp::max(self.size() * 2, requested_size);
        self.resize(new_size);
    }

    fn expand_exact(&mut self, additional: usize){
        self.resize(self.size() + additional);
    }

    fn resize(&mut self, new_size: usize);
}