use std::any::TypeId;
use std::{mem, ptr};
use std::marker::PhantomData;
use crate::any_value::{AnyValue, AnyValueCloneable, AnyValueMut, clone_into, copy_bytes, Unknown};
use crate::any_vec_raw::AnyVecRaw;
use super::any_vec_ptr::{IAnyVecPtr, IAnyVecRawPtr};
use crate::traits::{Cloneable, None, Trait};

pub trait Operation {
    type AnyVecPtr: IAnyVecRawPtr;
    type Type: 'static;

    fn any_vec_ptr(&self) -> Self::AnyVecPtr;

    fn bytes(&self) -> *const u8;

    /// Called after bytes move.
    fn consume(&mut self);
}

/// Temporary existing value in memory.
/// Data will be erased with TempValue destruction.
///
/// Have internal `&mut AnyVec`.
///
/// May do some postponed actions on consumption/destruction.
///
pub struct TempValue<Op: Operation, Traits: ?Sized + Trait = dyn None>{
    op: Op,
    phantom: PhantomData<Traits>
}
impl<Op: Operation, Traits: ?Sized + Trait> TempValue<Op, Traits>{
    pub(crate) fn new(op: Op) -> Self {
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

    #[inline]
    unsafe fn move_into(mut self, out: *mut u8) {
        copy_bytes(&self, out);
        self.op.consume();
        mem::forget(self);
    }
}

impl<Op: Operation, Traits: ?Sized + Trait> AnyValueMut for TempValue<Op, Traits>
    where Traits: Cloneable, Op::AnyVecPtr : IAnyVecPtr<Traits>
{}

impl<Op: Operation, Traits: ?Sized + Trait> AnyValueCloneable for TempValue<Op, Traits>
    where Traits: Cloneable, Op::AnyVecPtr : IAnyVecPtr<Traits>
{
    #[inline]
    unsafe fn clone_into(&self, out: *mut u8) {
        let any_vec = self.op.any_vec_ptr().any_vec().as_ref();
        clone_into(self, out, any_vec.clone_fn());
    }
}

impl<Op: Operation, Traits: ?Sized + Trait> Drop for TempValue<Op, Traits>{
    #[inline]
    fn drop(&mut self) {
        unsafe{
            let drop_fn = self.any_vec_raw().drop_fn();
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
        self.op.consume();
    }
}

unsafe impl<Op: Operation, Traits: ?Sized + Trait + Send> Send for TempValue<Op, Traits> {}
unsafe impl<Op: Operation, Traits: ?Sized + Trait + Sync> Sync for TempValue<Op, Traits> {}
