use std::any::TypeId;
use std::{mem, ptr};
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValueCloneable, copy_bytes, Unknown};
use crate::any_vec_raw::AnyVecRaw;
use crate::{AnyVec, copy_bytes_nonoverlapping};
use super::any_vec_ptr::{IAnyVecPtr, IAnyVecRawPtr};
use crate::traits::{Cloneable, EmptyTrait, Trait};

pub trait Operation {
    type AnyVecPtr: IAnyVecRawPtr;
    type Type: 'static;

    fn any_vec_ptr(&self) -> Self::AnyVecPtr;

    fn bytes(&self) -> *const u8;

    // TODO: rename somehow
    fn consume_op(&mut self);
}

/// Temporary existing value in memory.
/// Data will be erased with TempValue destruction.
///
/// Have internal `&mut AnyVec`.
///
/// May do some postponed actions on consumption/destruction.
///
pub struct TempValue<Op: Operation, Traits: ?Sized + Trait = dyn EmptyTrait>{
    op: Op,
    phantom: PhantomData<Traits>
}
impl<Op: Operation, Traits: ?Sized + Trait> TempValue<Op, Traits>{
    pub fn new(op: Op) -> Self {
        Self{op, phantom: PhantomData}
    }

    fn any_vec_raw(&self) -> &AnyVecRaw{
        unsafe{ self.op.any_vec_ptr().any_vec_raw().as_ref() }
    }
}

impl<Op: Operation, Traits: ?Sized + Trait> AnyValue for TempValue<Op, Traits>{
    type Type = Op::Type;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        let typeid = TypeId::of::<Op::Type>();
        if typeid == TypeId::of::<Unknown>(){
            self.any_vec_raw().element_typeid()
        } else {
            typeid
        }
    }

    #[inline]
    fn size(&self) -> usize {
        if Unknown::is::<Op::Type>() {
            self.any_vec_raw().element_layout().size()
        } else{
            mem::size_of::<Op::Type>()
        }
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.op.bytes()
    }

    unsafe fn consume_into(mut self, out: *mut u8)
    {
        copy_bytes(&self, out);
        self.op.consume_op();
        mem::forget(self);
    }
}

impl<Op: Operation, Traits: ?Sized + Trait> AnyValueCloneable for TempValue<Op, Traits>
    where Traits: Cloneable, Op::AnyVecPtr : IAnyVecPtr<Traits>
{
    unsafe fn clone_into(&self, out: *mut u8) {
        let any_vec = self.op.any_vec_ptr().any_vec().as_ref();
        if let Some(clone_fn) = any_vec.clone_fn(){
            (clone_fn)(self.bytes(), out, 1);
        } else {
            copy_bytes(self, out);
        }
    }
}

impl<Op: Operation, Traits: ?Sized + Trait> Drop for TempValue<Op, Traits>{
    #[inline]
    fn drop(&mut self) {
        unsafe{
            let drop_fn = self.any_vec_raw().drop_fn;
            let element = self.op.bytes() as *mut u8;

            // compile-time check
            if Unknown::is::<Op::Type>() {
                if let Some(drop_fn) = drop_fn{
                    (drop_fn)(element, 1);
                }
            } else {
                ptr::drop_in_place(element as *mut Op::Type);
            }
        }
        self.op.consume_op();
    }
}

unsafe impl<Op: Operation, Traits: ?Sized + Trait + Send> Send for TempValue<Op, Traits> {}
unsafe impl<Op: Operation, Traits: ?Sized + Trait + Sync> Sync for TempValue<Op, Traits> {}
