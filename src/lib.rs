//! Type erased vector [`AnyVec`]. Allow to store elements of the same type.
//! Have same performance and *operations* as [`std::vec::Vec`].
//!
//! You can downcast type erased [`AnyVec`] to concrete [`AnyVecTyped<Element>`] with `downcast`-family.
//! Or use [`AnyVec`] type erased operations, which works with [`AnyValue`].
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
//! N.B. [`AnyVecTyped`] operations may be somewhat faster, due to the fact that
//! compiler able to do better optimisation with full type knowledge.
//!
//! # Send, Sync, Clone
//!
//! You can make [`AnyVec`] [`Send`]able, [`Sync`]able, [`Cloneable`]:
//!
//! [`Cloneable`]: traits::Cloneable
//!
//!```rust
//! use any_vec::AnyVec;
//! use any_vec::traits::*;
//! let v1: AnyVec<dyn Cloneable + Sync + Send> = AnyVec::new::<String>();
//! let v2 = v1.clone();
//! ```
//! This constraints will be applied compiletime to element type:
//!```compile_fail
//! # use any_vec::AnyVec;
//! # use std::rc::Rc;
//! let v1: AnyVec<dyn Sync + Send> = AnyVec::new::<Rc<usize>>();
//!```
//!
//! # LazyClone
//!
//! Whenever possible, [`any_vec`] types implement [`AnyValueCloneable`], which
//! can work with [`LazyClone`]:
//!
//! [`any_vec`]: crate
//! [`AnyValueCloneable`]: any_value::AnyValueCloneable
//! [`LazyClone`]: any_value::LazyClone
//!
//!```rust
//! # use any_vec::any_value::{AnyValueCloneable, AnyValueWrapper};
//! # use any_vec::AnyVec;
//! # use any_vec::traits::*;
//! let mut v1: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
//! v1.push(AnyValueWrapper::new(String::from("0")));
//!
//! let mut v2: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
//! let e = v1.swap_remove(0);
//! v2.push(e.lazy_clone());
//! v2.push(e.lazy_clone());
//! ```

mod any_vec;
mod clone_type;
mod any_vec_ptr;
mod any_vec_raw;
mod any_vec_typed;
mod iter;
mod refs;

pub use crate::any_vec::{AnyVec, AnyVecMut, AnyVecRef, SatisfyTraits, traits};
pub use any_vec_typed::AnyVecTyped;
pub use iter::{ElementIterator, Iter, IterRef, IterMut};

pub mod any_value;
pub mod ops;
pub mod element;

use std::ptr;
use std::ops::{Bound, Range, RangeBounds};

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

// This is faster then ptr::copy,
// when count is runtime value, and count is small.
#[inline]
unsafe fn copy_bytes(src: *const u8, dst: *mut u8, count: usize){
    // MIRI hack
    if cfg!(miri)
        || count >= 128
    {
        ptr::copy(src, dst, count);
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

#[inline]
fn into_range(
    len: usize,
    range: impl RangeBounds<usize>
) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Included(i) => *i,
        Bound::Excluded(i) => *i + 1,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(i) => *i + 1,
        Bound::Excluded(i) => *i,
        Bound::Unbounded => len,
    };
    assert!(start <= end);
    assert!(end <= len);
    start..end
}
