mod heap;

pub use heap::Heap;

pub(crate) type Default = Heap;

use std::alloc::Layout;
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
        let requested_capacity = self.size() + additional;
        let mut new_capacity = self.size();
        loop{
            new_capacity *= 2;
            if new_capacity >= requested_capacity{
                break;
            }
        }
        self.resize(new_capacity);
    }

    fn expand_exact(&mut self, additional: usize){
        self.resize(self.size() + additional);
    }

    fn resize(&mut self, new_size: usize);
}