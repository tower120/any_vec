use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::mem::{forget, MaybeUninit};
use std::ptr;
use std::ptr::NonNull;
use crate::{AnyValue, AnyVec, copy_bytes_nonoverlapping, UnknownType};
use crate::any_value_tmp2::Impl;


pub struct SwapRemove2<'a, T: 'static = UnknownType>{
    pub(crate) any_vec: &'a mut AnyVec,
    pub(crate) index: usize,
    pub(crate) phantom: PhantomData<&'a mut T>
}

impl<'a, T: 'static> Impl for SwapRemove2<'a, T>{
    type Type = T;

    #[inline]
    fn any_vec(&self) -> &AnyVec {
        self.any_vec
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(&mut self, f: F) {
        let element_size = self.any_vec.element_layout().size();
        let element = self.any_vec.mem.as_ptr().add(element_size * self.index);

        // 1. Consume
        f(NonNull::new_unchecked(element));

        // 2. overwrite with last element
        let last_index = self.any_vec.len - 1;
        let last_element = self.any_vec.mem.as_ptr().add(element_size * last_index);
        if self.index != last_index {
            copy_bytes_nonoverlapping
            //std::ptr::copy_nonoverlapping
                (last_element, element, element_size);
        }

        // 3. shrink len
        self.any_vec.len -= 1;
    }
}




pub struct SwapRemove<'a, T: 'static = UnknownType>{
    pub(crate) any_vec: &'a mut AnyVec,
    pub(crate) index: usize,
    pub(crate) phantom: PhantomData<&'a mut T>
}

fn drop_value(any_vec: &mut AnyVec, value: impl AnyValue){
    unsafe{
        value.consume_bytes(|element|{
            any_vec.drop_element(element.as_ptr(), 1);
        });
    }
}

impl<'a, T> AnyValue for SwapRemove<'a, T>{
    #[inline]
    fn value_typeid(&self) -> TypeId {
        let typeid = TypeId::of::<T>();
        if typeid == TypeId::of::<UnknownType>(){
            self.any_vec.element_typeid()
        } else {
            typeid
        }
    }

    #[inline]
    unsafe fn consume_bytes<F: FnOnce(NonNull<u8>)>(self, f: F) {
        let element_size = self.any_vec.element_layout().size();
        let element = self.any_vec.mem.as_ptr().add(element_size * self.index);
        // 1. Consume
        f(NonNull::new_unchecked(element));

        // 2. overwrite with last element
        let last_index = self.any_vec.len - 1;
        let last_element = self.any_vec.mem.as_ptr().add(element_size * last_index);
        if self.index != last_index {
            //copy_bytes_nonoverlapping
            std::ptr::copy_nonoverlapping
                (last_element, element, element_size);
        }

        // 3. shrink len
        self.any_vec.len -= 1;
    }
}

impl<'a, T> Drop for SwapRemove<'a, T>{
    #[inline]
    fn drop(&mut self) {
    // unsafe{
    //     self.consume_bytes(|element|{
    //         self.any_vec.drop_element(element.as_ptr(), 1);
    //     });
    // }
    }
}