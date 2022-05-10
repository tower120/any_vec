use std::{mem, ptr};
use std::alloc::{alloc, dealloc, Layout, realloc};
use std::any::TypeId;
use std::cmp::max;
use std::mem::{MaybeUninit, size_of};
use std::ptr::{null_mut};

const MIN_CAPACITY: usize = 2;

/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// *`T:'static` due to TypeId requirements*
pub struct RawVec{
    mem: *mut u8,
    capacity: usize,        // in elements
    len: usize,             // in elements
    element_size: usize,    // aligned
    type_id:  TypeId,       // purely for safety checks
    drop_fn: fn(ptr: *mut u8, len: usize)
}

impl RawVec{
    #[inline]
    pub fn new<T:'static>() -> Self {
        Self{
            mem: unsafe{alloc(Layout::from_size_align_unchecked(
                size_of::<T>() * MIN_CAPACITY, 1
            ))},
            capacity: MIN_CAPACITY,
            len: 0,
            element_size: size_of::<T>(),   // aligned!
            type_id: TypeId::of::<T>(),
            drop_fn: |mut ptr: *mut u8, len: usize|{
                // compile time check
                if !mem::needs_drop::<T>(){
                    return;
                }

                for _ in 0..len{
                    unsafe{
                        ptr::drop_in_place(ptr as *mut T);
                        ptr = ptr.add(mem::size_of::<T>());
                    }
                }
            }
        }
    }

    fn set_capacity(&mut self, mut new_capacity: usize){
        assert!(self.len <= new_capacity);
        new_capacity = max(MIN_CAPACITY, new_capacity);
        unsafe{
            let new_mem = realloc(self.mem, Layout::from_size_align_unchecked(
                self.element_size * self.capacity, 1
            ),  self.element_size * new_capacity);

            self.mem = new_mem;
            self.capacity = new_capacity;
        }
    }

    fn grow(&mut self){
        self.set_capacity(self.capacity * 2);
    }

    /// Pushes one element represented as byte slice.
    ///
    /// # Safety
    /// This is highly unsafe, due to the number of invariants that aren’t checked:
    /// * element_bytes must belong to Element.
    /// * element_bytes must be aligned.
    /// * Element must be "forgot".
    #[inline]
    pub unsafe fn push_bytes(&mut self, element_bytes: &[u8]){
        debug_assert!(element_bytes.len() == self.element_size);

        if self.len == self.capacity{
            self.grow();
        }

        let new_element = self.mem.add(self.element_size * self.len);
        ptr::copy_nonoverlapping(element_bytes.as_ptr(), new_element, self.element_size);

        self.len += 1;
    }

    /// # Safety
    ///
    /// Unsafe, because type match not checked.
    #[inline]
    pub unsafe fn push_unchecked<T>(&mut self, value: T){
        let bytes = std::slice::from_raw_parts(
            (&value as *const T) as *const u8,
            std::mem::size_of::<T>(),
        );
        mem::forget(value);
        self.push_bytes(bytes);
    }

    /// # Panics
    ///
    /// Panics if type mismatch.
    #[inline]
    pub fn push<T:'static>(&mut self, value: T){
        assert_eq!(TypeId::of::<T>(), self.type_id);
        unsafe{
            self.push_unchecked(value);
        }
    }

    /// drop element, if out is null.
    #[inline]
    unsafe fn swap_take_bytes_impl(&mut self, index: usize, out: *mut u8) {
        assert!(index < self.len);

        // 1. drop element at index
        let element = self.mem.add(self.element_size * index);
        if !out.is_null() {
            ptr::copy_nonoverlapping(element, out, self.element_size);
        } else {
            (self.drop_fn)(element, 1);
        }

        // 2. move element
        let last_index = self.len - 1;
        if index != last_index {
            let last_element = self.mem.add(self.element_size * last_index);
            ptr::copy_nonoverlapping(last_element, element, self.element_size);
        }

        // 3. shrink len
        self.len -= 1;
    }

    /// # Panics
    ///
    /// Panics if index out of bounds.
    pub fn swap_remove(&mut self, index: usize) {
        unsafe{
            self.swap_take_bytes_impl(index, null_mut());
        }
    }

    /// Same as [`swap_remove`], but copy removed element as bytes to `out`.
    ///
    /// # Safety
    /// * It is your responsibility to properly drop element.
    /// * `out` must have at least [`size_of_element`] bytes.
    ///
    /// # Panics
    /// Panics if index out of bounds
    ///
    /// [`swap_remove`]: Self::swap_remove
    /// [`size_of_element`]: Self::size_of_element
    pub unsafe fn swap_take_bytes(&mut self, index: usize, out: *mut u8){
        self.swap_take_bytes_impl(index, out);
    }

    /// Same as [`swap_remove`], but return removed element.
    ///
    /// [`swap_remove`]: Self::swap_remove
    ///
    /// # Panics
    /// Panics if index out of bounds.
    /// Panics if type mismatch.
    pub fn swap_take<T:'static>(&mut self, index: usize) -> T {
        assert_eq!(TypeId::of::<T>(), self.type_id);

        let mut out = MaybeUninit::<T>::uninit();
        unsafe{
            self.swap_take_bytes_impl(index, out.as_mut_ptr() as *mut u8);
            out.assume_init()
        }
    }


    #[inline]
    pub fn clear(&mut self){
        (self.drop_fn)(self.mem, self.len);
        self.len = 0;
    }

    #[inline]
    pub unsafe fn as_slice_unchecked<T>(&self) -> &[T]{
        std::slice::from_raw_parts(
            self.mem as *const T,
            self.len,
        )
    }

    /// # Panics
    ///
    /// Panics if type mismatch.
    #[inline]
    pub fn as_slice<T:'static>(&self) -> &[T]{
        assert_eq!(TypeId::of::<T>(), self.type_id);
        unsafe{
            self.as_slice_unchecked()
        }
    }

    #[inline]
    pub unsafe fn as_mut_slice_unchecked<T:'static>(&self) -> &mut[T]{
        std::slice::from_raw_parts_mut(
            self.mem as *mut T,
            self.len,
        )
    }

    /// # Panics
    ///
    /// Panics if type mismatch.
    #[inline]
    pub fn as_mut_slice<T:'static>(&self) -> &mut[T]{
        assert_eq!(TypeId::of::<T>(), self.type_id);
        unsafe{
            self.as_mut_slice_unchecked::<T>()
        }
    }

    pub fn type_id(&self) -> TypeId{
        self.type_id
    }

    /// In bytes. Aligned. `size_of::<T>()`
    pub fn size_of_element(&self) -> usize {
        self.element_size
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for RawVec{
    fn drop(&mut self) {
        self.clear();
        unsafe{
            dealloc(self.mem, Layout::from_size_align_unchecked(
                self.element_size * self.capacity, 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::forget;
    use itertools::assert_equal;
    use crate::v2::RawVec;

    unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
        std::slice::from_raw_parts(
            (p as *const T) as *const u8,
            std::mem::size_of::<T>(),
        )
    }

    struct S{i:usize}
    impl Drop for S{
        fn drop(&mut self) {
            println!("Drop {}",self.i);
        }
    }

    #[test]
    fn drop_test() {
        let mut raw_vec = RawVec::new::<S>();
        unsafe{
            raw_vec.push_unchecked(S{i:1});
            raw_vec.push_unchecked(S{i:2});
            raw_vec.push_unchecked(S{i:3});
        }
        unsafe{
            assert_equal(raw_vec.as_slice_unchecked::<S>().iter().map(|s|s.i), [1, 2, 3]);
        }
    }

    #[test]
    fn it_works() {
        let mut raw_vec = RawVec::new::<String>();

        unsafe{
            let str1 = "Hello".to_string();
            raw_vec.push_bytes(any_as_u8_slice(&str1));
            forget(str1);

            let str2 = " to ".to_string();
            raw_vec.push_bytes(any_as_u8_slice(&str2));
            forget(str2);

            let str3 = "world".to_string();
            raw_vec.push_bytes(any_as_u8_slice(&str3));
            forget(str3);
        }

        unsafe{
            assert_equal(raw_vec.as_slice_unchecked::<String>(), ["Hello", " to ", "world"]);
        }
    }

    #[test]
    fn zero_size_type_test() {
        struct Empty{}
        let mut raw_vec = RawVec::new::<Empty>();
        unsafe{
            raw_vec.push_unchecked(Empty{});
            raw_vec.push_unchecked(Empty{});
            raw_vec.push_unchecked(Empty{});
        }
    }

    #[test]
    fn swap_remove_test() {
        let mut raw_vec = RawVec::new::<String>();
        raw_vec.push(String::from("0"));
        raw_vec.push(String::from("1"));
        raw_vec.push(String::from("2"));
        raw_vec.push(String::from("3"));
        raw_vec.push(String::from("4"));

        {
            let e: String = raw_vec.swap_take(1);
            assert_eq!(e, String::from("1"));
            assert_equal(raw_vec.as_slice::<String>(), &[
                String::from("0"),
                String::from("4"),
                String::from("2"),
                String::from("3"),
            ]);
        }

        {
            raw_vec.swap_remove(2);
            assert_equal(raw_vec.as_slice::<String>(), &[
                String::from("0"),
                String::from("4"),
                String::from("3"),
            ]);
        }
    }
}