use std::any::TypeId;
use std::marker::PhantomData;
use std::{mem, ptr};
use std::ptr::NonNull;
use crate::{AnyValue, AnyVec, UnknownType};

pub trait Impl{
    type Type: 'static;
    fn any_vec(&self) -> &AnyVec;
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(&mut self, f: F);
}

// AnyVecMutator ??
pub struct AnyValueTemp2<I: Impl>(pub(crate) I);

impl<I: Impl> AnyValue for AnyValueTemp2<I>{
    #[inline]
    fn value_typeid(&self) -> TypeId {
        let typeid = TypeId::of::<I::Type>();
        if typeid == TypeId::of::<UnknownType>(){
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

impl<I: Impl> Drop for AnyValueTemp2<I>{
    #[inline]
    fn drop(&mut self) {
    unsafe{
        let drop_fn = self.0.any_vec().drop_fn;
        self.0.consume_bytes(|element|{
            // compile-time check
            if TypeId::of::<I::Type>() == TypeId::of::<UnknownType>(){
                if let Some(drop_fn) = drop_fn{
                    (drop_fn)(element.as_ptr(), 1);
                }
            } else {
                ptr::drop_in_place(element.as_ptr() as *mut  I::Type);
            }
        });
    }
    }
}