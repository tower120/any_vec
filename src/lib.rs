//! Type erased vector [`AnyVec`]. Allow to store elements of the same type.
//! Have same performance and *operations* as [`std::vec::Vec`].
//!
//! You can downcast type erased [`AnyVec`] to concrete [`AnyVecTyped<Element>`] with `downcast`-family.
//! Or use [`AnyVec`]'s type erased operations, which operate on `[u8]` byte basis.
//!
//! ```rust
//!     use any_vec::AnyVec;
//!     let mut vec: AnyVec = AnyVec::new::<String>();
//!     {
//!         // Typed operations.
//!         let mut vec = vec.downcast_mut::<String>().unwrap();
//!         vec.push(String::from("0"));
//!         vec.push(String::from("1"));
//!         vec.push(String::from("2"));
//!     }
//!
//!     let mut other_vec: AnyVec = AnyVec::new::<String>();
//!     // Fully type erased element move from one vec to another
//!     // without intermediate mem-copies.
//!     //
//!     // Equivalent to:
//!     //
//!     // let element = vec.swap_remove(0);
//!     // other.push(element);
//!     unsafe{
//!         let element: &mut[u8] = other_vec.push_uninit();    // allocate element
//!         vec.swap_remove_into(0, element);                   // swap_remove
//!     }
//!
//!     // Output 2 1
//!     for s in vec.downcast_ref::<String>().unwrap().as_slice(){
//!         println!("{}", s);
//!     }
//!
//!```
//!
//! [`AnyVec`] is [`Send`]able if it's elements are.
//! [`AnyVec`] is [`Sync`]able if it's elements are.
//!

mod any_vec;
mod any_vec_typed;
mod any_vec_mut;
mod any_vec_ref;

pub use crate::any_vec::AnyVec;
pub use any_vec_typed::AnyVecTyped;
pub use any_vec_mut::AnyVecMut;
pub use any_vec_ref::AnyVecRef;

use std::ptr;

// This is faster then ptr::copy_nonoverlapping,
// when count is runtime value, and count is small.
#[inline]
unsafe fn copy_bytes_nonoverlapping(src: *const u8, dst: *mut u8, count: usize){
    // MIRI hack
    if cfg!(miri) {
        ptr::copy_nonoverlapping(src, dst, count);
        return;
    }

    for i in 0..count{
        *dst.add(i) = *src.add(i);
    }
}

// same as copy_bytes_nonoverlapping but for swap_nonoverlapping.
#[inline]
unsafe fn swap_bytes_nonoverlapping(src: *mut u8, dst: *mut u8, count: usize){
    // MIRI hack
    if cfg!(miri) {
        let mut tmp = Vec::<u8>::new();
        tmp.resize(count, 0);

        // src -> tmp
        ptr::copy_nonoverlapping(src, tmp.as_mut_ptr(), count);
        // dst -> src
        ptr::copy_nonoverlapping(dst, src, count);
        // tmp -> dst
        ptr::copy_nonoverlapping(tmp.as_ptr(), dst, count);

        return;
    }

    for i in 0..count{
        let src_pos = src.add(i);
        let dst_pos = dst.add(i);

        let tmp = *src_pos;
        *src_pos = *dst_pos;
        *dst_pos = tmp;
    }
}
