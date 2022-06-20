mod heap;
mod stack;

pub use heap::Heap;
pub use stack::Stack;

pub(crate) type Default = Heap;

use std::alloc::Layout;
use std::cmp;
use std::ptr::NonNull;

/// This is [`Mem`] builder.
///
/// It can be stateful. You can use it like Allocator.
pub trait MemBuilder: Clone {
    type Mem: Mem;
    fn build(&mut self, element_layout: Layout, capacity: usize) -> Self::Mem;
}

/// Interface for [`AnyVec`] memory chunk.
///
/// Responsible for allocation, dealocation, reallocation of the memory chunk.
/// Constructed through `Mem::Builder`.
///
/// _`Mem` is fixed capacity memory. Implement [`MemResizable`] if you want it
/// to be resizable._
pub trait Mem{
    fn as_ptr(&self) -> *const u8;

    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Aligned.
    fn element_layout(&self) -> Layout;

    /// In elements.
    fn size(&self) -> usize;

    /// AnyVec implementation need this. Should be in MemResizable.
    fn expand(&mut self, additional: usize){
        drop(additional);
        panic!("Can't change capacity!");

        /*let requested_size = self.size() + additional;
        let new_size = cmp::max(self.size() * 2, requested_size);
        self.resize(new_size);*/
    }
}

/// Marker trait for resizable ['Mem'].
pub trait MemResizable: Mem{
    fn expand_exact(&mut self, additional: usize){
        self.resize(self.size() + additional);
    }

    /// Do panic if can't resize.
    fn resize(&mut self, new_size: usize);
}