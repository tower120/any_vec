use std::any::TypeId;
use std::{mem, ptr};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::DerefMut;
use std::ptr::{drop_in_place, null_mut};

pub trait IAnyValue{
    fn type_id(&self) -> TypeId;
    fn downcast<T: 'static>(self) -> T;
    unsafe fn consume_bytes<F: FnOnce(*mut u8)>(self, f: F);
}

// AnyValuePtr / AnyValueRef ?
// AnyValueTemp - temporary exists in memory value, data will be erased with AnyValueTemp destruction.
// May do some postponed actions on destruction.
pub struct AnyValue<'a, DropFn: FnOnce(*mut u8)>{
    mem: *mut u8,        // TODO: NonNull everywhere
    typeid: TypeId,
    drop_fn: ManuallyDrop<DropFn>,
    phantom: PhantomData<&'a mut [u8]>
}

impl<'a, DropFn: FnOnce(*mut u8)> AnyValue<'a, DropFn>{
    pub unsafe fn from_raw_parts(mem: *mut u8, typeid: TypeId, drop_fn: DropFn) -> Self {
        Self{
            mem,
            typeid,
            drop_fn: ManuallyDrop::new(drop_fn),
            phantom: PhantomData
        }
    }
}

impl<'a, DropFn: FnOnce(*mut u8)> IAnyValue for AnyValue<'a, DropFn>{
    fn type_id(&self) -> TypeId {
        self.typeid
    }

    fn downcast<T: 'static>(self) -> T{
        assert_eq!(self.typeid, TypeId::of::<T>());
        unsafe{
            let value = ptr::read(self.mem.cast::<T>());
            let drop_fn = ptr::read(&*self.drop_fn);
            (drop_fn)(null_mut());
            mem::forget(self);
            value
        }
    }

    unsafe fn consume_bytes<F: FnOnce(*mut u8)>(self, f: F) {
        f(self.mem);
        let drop_fn = ptr::read(&*self.drop_fn);
        (drop_fn)(null_mut());
        mem::forget(self);
    }
}

impl<'a, DropFn: FnOnce(*mut u8)> Drop for AnyValue<'a, DropFn>{
    fn drop(&mut self) {
        unsafe{
            let drop_fn = ptr::read(&*self.drop_fn);
            (drop_fn)(self.mem);
        }
    }
}