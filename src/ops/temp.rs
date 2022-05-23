use std::any::TypeId;
use std::{mem, ptr};
use std::ptr::NonNull;
use crate::{AnyVec, Unknown};
use crate::any_value::AnyValue;

pub trait Operation {
    type Type: 'static;
    fn any_vec(&self) -> &AnyVec;
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(&mut self, f: F);
}

/// Temporary existing value in memory.
/// Data will be erased with AnyValueTemp destruction.
///
/// Have internal `&mut AnyVec`.
///
/// May do some postponed actions on consumption/destruction.
///
pub struct AnyValueTemp<Op: Operation>(pub(crate) Op);

impl<Op: Operation> AnyValue for AnyValueTemp<Op>{
    type Type = Op::Type;

    #[inline]
    fn value_typeid(&self) -> TypeId {
        let typeid = TypeId::of::<Op::Type>();
        if typeid == TypeId::of::<Unknown>(){
            self.0.any_vec().element_typeid()
        } else {
            typeid
        }
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(mut self, f: F) {
        self.0.consume_bytes(f);
        mem::forget(self);
    }
}

impl<Op: Operation> Drop for AnyValueTemp<Op>{
    #[inline]
    fn drop(&mut self) {
    unsafe{
        let drop_fn = self.0.any_vec().drop_fn;
        self.0.consume_bytes(|element|{
            // compile-time check
            if Unknown::is::<Op::Type>() {
                if let Some(drop_fn) = drop_fn{
                    (drop_fn)(element.as_ptr(), 1);
                }
            } else {
                ptr::drop_in_place(element.as_ptr() as *mut  Op::Type);
            }
        });
    }
    }
}