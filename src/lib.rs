use std::{mem, ptr};
use std::alloc::{alloc, dealloc, Layout, realloc};
use std::any::TypeId;
use std::cmp::max;
use std::mem::{MaybeUninit};
use std::ptr::{null_mut};

const MIN_CAPACITY: usize = 2;

// Never touched
static mut ZST_ARRAY: [u8;1] = [0];

/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// *`T:'static` due to TypeId requirements*
pub struct AnyVec {
    mem: *mut u8,
    capacity: usize,        // in elements
    len: usize,             // in elements
    element_layout: Layout, // size is aligned
    type_id: TypeId,        // purely for safety checks
    drop_fn: fn(ptr: *mut u8, len: usize)
}

impl AnyVec {
    pub fn new<T:'static>() -> Self {
        Self::with_capacity::<T>(MIN_CAPACITY)
    }

    pub fn with_capacity<T:'static>(mut capacity: usize) -> Self {
        capacity = max(capacity, MIN_CAPACITY);

        let element_layout = Layout::new::<T>();
        let mem = unsafe {
            if element_layout.size() != 0 {
                alloc(Layout::from_size_align_unchecked(
                    element_layout.size() * capacity, element_layout.align()
                ))
            } else {
                &mut ZST_ARRAY as *mut u8
            }
        };
        Self{
            mem,
            capacity,
            len: 0,
            element_layout,
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
        // Never cut
        debug_assert!(self.len <= new_capacity);

        new_capacity = max(MIN_CAPACITY, new_capacity);
        unsafe{
            if self.element_layout.size() != 0 {
                let mem_layout = Layout::from_size_align_unchecked(
                    self.element_layout.size() * self.capacity, self.element_layout.align()
                );
                // mul carefully, to prevent overflow.
                let new_mem_size = self.element_layout.size().checked_mul(new_capacity).unwrap();
                self.mem = realloc(self.mem, mem_layout,new_mem_size);
            }
            self.capacity = new_capacity;
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self){
        self.set_capacity(self.capacity * 2);
    }

    /// Pushes one element without actually writing anything.
    ///
    /// Return byte slice, that must be filled with element data.
    ///
    /// # Safety
    /// This is highly unsafe, due to the number of invariants that aren’t checked:
    /// * returned byte slice must be written with actual Element bytes.
    /// * Element bytes must be aligned.
    /// * Element must be "forgotten".
    #[inline]
    pub unsafe fn push_uninit(&mut self) -> &mut[u8] {
        if self.len == self.capacity{
            self.grow();
        }

        let new_element = self.mem.add(self.element_layout.size() * self.len);
        self.len += 1;

        std::slice::from_raw_parts_mut(
            new_element,
            self.element_layout.size(),
        )
    }

    /// Marginally faster [`push`] version. *(See benches/push)*
    ///
    /// [`push`]: Self::push
    ///
    /// # Safety
    ///
    /// Unsafe, because type not checked.
    #[inline]
    pub unsafe fn push_unchecked<T>(&mut self, value: T){
        ptr::write(self.push_uninit().as_mut_ptr() as *mut T, value);
    }

    /// Appends an element to the back of a collection.
    ///
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
        let element = self.mem.add(self.element_layout.size() * index);
        if !out.is_null() {
            ptr::copy_nonoverlapping(element, out, self.element_layout.size());
        } else {
            (self.drop_fn)(element, 1);
        }

        // 2. move element
        let last_index = self.len - 1;
        if index != last_index {
            let last_element = self.mem.add(self.element_layout.size() * last_index);
            ptr::copy_nonoverlapping(last_element, element, self.element_layout.size());
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
    /// * It is your responsibility to properly drop `out` element.
    ///
    /// # Panics
    /// * Panics if index out of bounds.
    /// * Panics if out len does not match element size.
    ///
    /// [`swap_remove`]: Self::swap_remove
    pub unsafe fn swap_take_bytes_into(&mut self, index: usize, out: &mut[u8]){
        assert_eq!(out.len(), self.element_layout.size());
        self.swap_take_bytes_impl(index, out.as_mut_ptr());
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

    /// Element TypeId
    pub fn element_typeid(&self) -> TypeId{
        self.type_id
    }

    /// Element Layout
    pub fn element_layout(&self) -> Layout {
        self.element_layout
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for AnyVec {
    fn drop(&mut self) {
        self.clear();
        if self.element_layout.size() != 0 {
            unsafe{
                dealloc(self.mem, Layout::from_size_align_unchecked(
                    self.element_layout.size() * self.capacity, self.element_layout.align()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::forget;
    use itertools::assert_equal;
    use crate::AnyVec;

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
        let mut raw_vec = AnyVec::new::<S>();
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
        let mut raw_vec = AnyVec::new::<String>();

        unsafe{
            let str1 = "Hello".to_string();
            raw_vec.push_uninit().copy_from_slice(any_as_u8_slice(&str1));
            forget(str1);

            let str2 = " to ".to_string();
            raw_vec.push_uninit().copy_from_slice(any_as_u8_slice(&str2));
            forget(str2);

            let str3 = "world".to_string();
            raw_vec.push_uninit().copy_from_slice(any_as_u8_slice(&str3));
            forget(str3);
        }

        unsafe{
            assert_equal(raw_vec.as_slice_unchecked::<String>(), ["Hello", " to ", "world"]);
        }
    }

    #[test]
    pub fn push_with_capacity_test(){
        const SIZE: usize = 10000;
        let mut vec = AnyVec::with_capacity::<usize>(SIZE);
        for i in 0..SIZE{
            vec.push(i);
        }

        assert_equal(vec.as_slice::<usize>().iter().copied(), 0..SIZE);
    }

    #[test]
    fn zero_size_type_test() {
        struct Empty{}
        let mut raw_vec = AnyVec::new::<Empty>();
        unsafe{
            raw_vec.push_unchecked(Empty{});
            raw_vec.push_unchecked(Empty{});
            raw_vec.push_unchecked(Empty{});
        }

        let mut i = 0;
        for _ in raw_vec.as_mut_slice::<Empty>(){
            i += 1;
        }
        assert_eq!(i, 3);
    }

    #[test]
    fn swap_remove_test() {
        let mut raw_vec = AnyVec::new::<String>();
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

    #[test]
    fn type_erased_move_test() {
        let mut raw_vec = AnyVec::new::<String>();
        raw_vec.push(String::from("0"));
        raw_vec.push(String::from("1"));
        raw_vec.push(String::from("2"));
        raw_vec.push(String::from("3"));
        raw_vec.push(String::from("4"));

        let mut other_vec = AnyVec::new::<String>();
        unsafe {
            let element = other_vec.push_uninit();
            raw_vec.swap_take_bytes_into(2, element);
        }

        assert_equal(other_vec.as_slice::<String>(), &[
            String::from("2"),
        ]);

        assert_equal(raw_vec.as_slice::<String>(), &[
            String::from("0"),
            String::from("1"),
            String::from("4"),
            String::from("3"),
        ]);
    }
}