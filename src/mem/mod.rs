mod heap;

pub use heap::Heap;

pub(crate) type Default = Heap;

use std::alloc::Layout;
use std::cmp;
use std::ptr::NonNull;

/// Interface for [`AnyVec`] memory chunk.
///
/// Responsible for allocation, dealocation, reallocation of the memory chunk.
/// Constructed through `Mem::Builder`.
pub trait Mem{
    type Builder: MemBuilder<Self>;

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

    /// Do panic if can't resize.
    fn resize(&mut self, new_size: usize);
}

/// This is [`Mem`] builder.
///
/// It can be stateful. You can use it like Allocator.
pub trait MemBuilder<M: Mem + ?Sized>: Clone{
    fn build(&mut self, element_layout: Layout, capacity: usize) -> M;
}