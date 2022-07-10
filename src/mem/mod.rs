mod heap;
mod stack;
mod stack_n;

pub use heap::Heap;
pub use stack::Stack;
pub use stack_n::StackN;

pub(crate) type Default = Heap;

use std::alloc::Layout;

/// This is [`Mem`] builder.
///
/// It can be stateful. You can use it like Allocator.
/// Making `MemBuilder` default constructible, allow to use [`AnyVec::new`], without that you
/// limited to [`AnyVec::new_in`].
///
/// [`AnyVec::new`]: crate::AnyVec::new
/// [`AnyVec::new_in`]: crate::AnyVec::new_in
pub trait MemBuilder: Clone {
    type Mem: Mem;
    fn build(&mut self, element_layout: Layout) -> Self::Mem;
}

/// This allows to use [`AnyVec::with_capacity`] with it.
///
/// [`AnyVec::with_capacity`]: crate::AnyVec::with_capacity
pub trait MemBuilderSizeable: MemBuilder{
    fn build_with_size(&mut self, element_layout: Layout, capacity: usize) -> Self::Mem;
}

/// Interface for [`AnyVec`] memory chunk.
///
/// Responsible for allocation, dealocation, reallocation of the memory chunk.
/// Constructed through [`MemBuilder`].
///
/// _`Mem` is fixed capacity memory. Implement [`MemResizable`] if you want it
/// to be resizable._
///
/// [`AnyVec`]: crate::AnyVec
pub trait Mem{
    fn as_ptr(&self) -> *const u8;

    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Aligned.
    fn element_layout(&self) -> Layout;

    /// In elements.
    fn size(&self) -> usize;

    /// Expand `Mem` size for **at least** `additional` more elements.
    /// Implementation encouraged to avoid frequent reallocations.
    ///
    /// # Notes
    ///
    /// Consider, that `expand` is in [`MemResizable`].
    /// Implement this only if your type [`MemResizable`].
    ///
    /// It's here, only due to technical reasons (see `AnyVecRaw::reserve`).
    ///
    /// # Panics
    ///
    /// Implementation may panic, if fail to allocate/reallocate memory.
    fn expand(&mut self, additional: usize){
        drop(additional);
        panic!("Can't change capacity!");

        /*let requested_size = self.size() + additional;
        let new_size = cmp::max(self.size() * 2, requested_size);
        self.resize(new_size);*/
    }
}

/// Resizable [`Mem`].
pub trait MemResizable: Mem{
    /// Expand `Mem` size for **exactly** `additional` more elements.
    /// Implementation encouraged to be as precise as possible with new memory size.
    ///
    /// # Panics
    ///
    /// Implementation may panic, if fail to allocate/reallocate memory.
    fn expand_exact(&mut self, additional: usize){
        self.resize(self.size() + additional);
    }

    /// Resize memory chunk to specified size.
    ///
    /// # Panics
    ///
    /// Implementation may panic, if fail to allocate/reallocate/deallocate memory.
    fn resize(&mut self, new_size: usize);
}