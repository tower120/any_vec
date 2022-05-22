//! Type erased vector [`AnyVec`]. Allow to store elements of the same type.
//! Have same performance and *operations* as [`std::vec::Vec`].
//!
//! You can downcast type erased [`AnyVec`] to concrete [`AnyVecTyped<Element>`] with `downcast`-family.
//! [`AnyVec`] type erased operations return [`AnyValue`], which you can use with other [`AnyVec`],
//! or cast to concrete type.
//!
//! [`AnyValue`]: any_value::AnyValue
//!
//! ```rust
//!     use any_vec::AnyVec;
//!     use any_vec::any_value::AnyValue;
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
//!     let element = vec.swap_remove(0);
//!     other_vec.push(element);
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

pub mod any_value;
pub mod ops;

use std::ptr;
use std::any::TypeId;

/// Marker for unknown type.
pub struct Unknown;
impl Unknown {
    #[inline]
    pub fn is<T:'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<Unknown>()
    }
}

// This is faster then ptr::copy_nonoverlapping,
// when count is runtime value, and count is small.
#[inline]
unsafe fn copy_bytes_nonoverlapping(src: *const u8, dst: *mut u8, count: usize){
    // MIRI hack
    if cfg!(miri)
//        || count >= 128
    {
        ptr::copy_nonoverlapping(src, dst, count);
        return;
    }

    for i in 0..count{
        *dst.add(i) = *src.add(i);
    }
}

// same as copy_bytes_nonoverlapping but for swap_nonoverlapping.
#[allow(dead_code)]
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
