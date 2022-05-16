mod any_vec;
mod any_vec_typed;
mod any_vec_mut;
mod any_vec_ref;

pub use any_vec::AnyVec;
pub use any_vec_typed::AnyVecTyped;
pub use any_vec_mut::AnyVecMut;
pub use any_vec_ref::AnyVecRef;

use std::{mem, ptr};
use std::alloc::{alloc, dealloc, Layout, realloc, handle_alloc_error};
use std::any::TypeId;
use std::mem::{MaybeUninit, size_of};
use std::ops::{Deref, DerefMut};

// This is faster then ptr::copy_nonoverlapping,
// when count is runtime value, and count is small.
#[inline]
unsafe fn copy_bytes(src: *const u8, dst: *mut u8, count: usize){
    for i in 0..count{
        *dst.add(i) = *src.add(i);
    }
}

// same as copy_bytes but for swap_nonoverlapping.
#[inline]
unsafe fn swap_bytes(src: *mut u8, dst: *mut u8, count: usize){
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
