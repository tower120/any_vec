use std::any::TypeId;
use std::{mem, ptr};
use std::ptr::NonNull;
use crate::any_value::{AnyValue, AnyValue2, AnyValue2Cloneable, Unknown};
use crate::any_vec_raw::AnyVecRaw;
use crate::{AnyVec, copy_bytes_nonoverlapping};
use crate::traits::{Cloneable, Trait};

pub trait Operation {
    type Traits: ?Sized + Trait;
    type Type: 'static;
    fn any_vec(&self) -> &AnyVec<Self::Traits>;

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
pub struct TempValue<Op: Operation>(pub(crate) Op);

impl<Op: Operation> AnyValue2 for TempValue<Op>{
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
    fn size(&self) -> usize {
        if Unknown::is::<Op::Type>() {
            self.0.any_vec().element_layout().size()
        } else{
            mem::size_of::<Op::Type>()
        }
    }

    #[inline]
    fn bytes(&self) -> *const u8 {
        self.0.bytes()
    }

    unsafe fn consume_into(mut self, out: *mut u8)
    {
        if !Unknown::is::<Self::Type>() {
            ptr::copy_nonoverlapping(
                self.bytes() as *const Self::Type,
                out as *mut Self::Type,
                1);
        } else {
            copy_bytes_nonoverlapping(
                self.bytes(),
                out,
                self.size());
        }

        self.0.consume_op();
        mem::forget(self);
    }
}

impl<Op: Operation> AnyValue2Cloneable for TempValue<Op>
    where Op::Traits: Cloneable
{
    unsafe fn clone_into(&self, out: *mut u8) {
        if let Some(clone_fn) = self.0.any_vec().clone_fn(){
            (clone_fn)(self.bytes(), out, 1);
        } else {
            // TODO: known type optimisation
            ptr::copy_nonoverlapping(self.bytes(), out, self.size());
        }
    }
}

impl<Op: Operation> Drop for TempValue<Op>{
    #[inline]
    fn drop(&mut self) {
        unsafe{
            let drop_fn = self.0.any_vec().raw.drop_fn;
            let element = self.0.bytes() as *mut u8;

            // compile-time check
            if Unknown::is::<Op::Type>() {
                if let Some(drop_fn) = drop_fn{
                    (drop_fn)(element, 1);
                }
            } else {
                ptr::drop_in_place(element as *mut Op::Type);
            }
        }
        self.0.consume_op();
    }
}

unsafe impl<Op: Operation> Send for TempValue<Op>
    where Op::Traits: Send {}
unsafe impl<Op: Operation> Sync for TempValue<Op>
    where Op::Traits: Sync {}
