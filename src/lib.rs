use std::{mem, ptr};
use std::alloc::{alloc, dealloc, Layout, realloc, handle_alloc_error};
use std::any::TypeId;
use std::mem::{MaybeUninit, size_of};
use std::ptr::{NonNull};

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


/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// *`T:'static` due to TypeId requirements*
pub struct AnyVec {
    mem: NonNull<u8>,
    capacity: usize,        // in elements
    len: usize,             // in elements
    element_layout: Layout, // size is aligned
    type_id: TypeId,        // purely for safety checks
    drop_fn: Option<fn(ptr: *mut u8, len: usize)>
}

impl AnyVec {
    pub fn new<T:'static>() -> Self {
        Self::with_capacity::<T>(0)
    }

    pub fn with_capacity<T:'static>(capacity: usize) -> Self {
        let mut this = Self{
            mem: NonNull::<u8>::dangling(),
            capacity: 0,
            len: 0,
            element_layout: Layout::new::<T>(),
            type_id: TypeId::of::<T>(),
            drop_fn:
                if !mem::needs_drop::<T>(){
                    None
                } else{
                    Some(|mut ptr: *mut u8, len: usize|{
                        for _ in 0..len{
                            unsafe{
                                ptr::drop_in_place(ptr as *mut T);
                                ptr = ptr.add(mem::size_of::<T>());
                            }
                        }
                    })
                }
        };
        this.set_capacity(capacity);
        this
    }

    fn set_capacity(&mut self, new_capacity: usize){
        // Never cut
        debug_assert!(self.len <= new_capacity);

        if self.capacity == new_capacity {
            return;
        }

        if self.element_layout.size() != 0 {
            unsafe{
                let mem_layout = Layout::from_size_align_unchecked(
                    self.element_layout.size() * self.capacity, self.element_layout.align()
                );

                self.mem =
                    if new_capacity == 0 {
                        dealloc(self.mem.as_ptr(), mem_layout);
                        NonNull::<u8>::dangling()
                    } else {
                        // mul carefully, to prevent overflow.
                        let new_mem_size = self.element_layout.size().checked_mul(new_capacity).unwrap();
                        let new_mem_layout = Layout::from_size_align_unchecked(
                            new_mem_size, self.element_layout.align()
                        );

                        if self.capacity == 0 {
                            // allocate
                            NonNull::new(alloc(new_mem_layout))
                        } else {
                            // reallocate
                            NonNull::new(realloc(
                                self.mem.as_ptr(), mem_layout,new_mem_size
                            ))
                        }
                        .unwrap_or_else(|| handle_alloc_error(new_mem_layout))
                    }
            }
        }
        self.capacity = new_capacity;
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self){
        self.set_capacity(
             if self.capacity == 0 {2} else {self.capacity * 2}
        );
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

        let new_element = self.mem.as_ptr().add(self.element_layout.size() * self.len);
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

    #[inline]
    fn drop_element(&mut self, ptr: *mut u8, len: usize){
        if let Some(drop_fn) = self.drop_fn{
            (drop_fn)(ptr, len);
        }
    }

    /// Type erased version of [`Vec::swap_remove`]. Due to this, does not return element.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) {
    unsafe{
        assert!(index < self.len);

        let element = self.mem.as_ptr().add(self.element_layout.size() * index);

        // 1. swap elements
        let last_index = self.len - 1;
        let last_element = self.mem.as_ptr().add(self.element_layout.size() * last_index);
        if index != last_index {
            if self.drop_fn.is_none(){
                copy_bytes(last_element, element, self.element_layout.size());
            } else {
                swap_bytes(last_element, element, self.element_layout.size());
            }
        }

        // 2. shrink len
        self.len -= 1;

        // 3. drop last
        self.drop_element(last_element, 1);
    }
    }

    /// drop element, if out is null.
    /// element_size as parameter - because it possible can be known at compile time
    #[inline]
    unsafe fn swap_take_bytes_impl(&mut self, index: usize, element_size: usize, out: *mut u8)
    {
        assert!(index < self.len);

        // 1. move out element at index
        let element = self.mem.as_ptr().add(element_size * index);
        ptr::copy_nonoverlapping(element, out, element_size);

        // 2. move element
        let last_index = self.len - 1;
        if index != last_index {
            let last_element = self.mem.as_ptr().add(element_size * last_index);
            ptr::copy_nonoverlapping(last_element, element, element_size);
        }

        // 3. shrink len
        self.len -= 1;
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
    #[inline]
    pub unsafe fn swap_take_bytes_into(&mut self, index: usize, out: &mut[u8]){
        assert_eq!(out.len(), self.element_layout.size());  // This allows compile time optimization!
        self.swap_take_bytes_impl(index, self.element_layout.size(), out.as_mut_ptr());
    }

    /// Same as [`swap_remove`], but return removed element.
    ///
    /// [`swap_remove`]: Self::swap_remove
    ///
    /// # Panics
    /// Panics if index out of bounds.
    /// Panics if type mismatch.
    #[inline]
    pub fn swap_take<T:'static>(&mut self, index: usize) -> T {
        assert_eq!(TypeId::of::<T>(), self.type_id);

        let mut out = MaybeUninit::<T>::uninit();
        unsafe{
            self.swap_take_bytes_impl(index, size_of::<T>(), out.as_mut_ptr() as *mut u8);
            out.assume_init()
        }
    }

    #[inline]
    pub fn clear(&mut self){
        let len = self.len;

        // Prematurely set the length to zero so that even if dropping the values panics users
        // won't be able to access the dropped values.
        self.len = 0;

        self.drop_element(self.mem.as_ptr(), len);
    }

    #[inline]
    pub unsafe fn as_slice_unchecked<T>(&self) -> &[T]{
        std::slice::from_raw_parts(
            self.mem.as_ptr().cast::<T>(),
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
    pub unsafe fn as_mut_slice_unchecked<T:'static>(&mut self) -> &mut[T]{
        std::slice::from_raw_parts_mut(
            self.mem.as_ptr().cast::<T>(),
            self.len,
        )
    }

    /// # Panics
    ///
    /// Panics if type mismatch.
    #[inline]
    pub fn as_mut_slice<T:'static>(&mut self) -> &mut[T]{
        assert_eq!(TypeId::of::<T>(), self.type_id);
        unsafe{
            self.as_mut_slice_unchecked::<T>()
        }
    }

    /// Element TypeId
    #[inline]
    pub fn element_typeid(&self) -> TypeId{
        self.type_id
    }

    /// Element Layout
    #[inline]
    pub fn element_layout(&self) -> Layout {
        self.element_layout
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for AnyVec {
    fn drop(&mut self) {
        self.clear();
        self.set_capacity(0);
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

        raw_vec.swap_remove(2);
        assert_equal(raw_vec.as_slice::<String>(), &[
            String::from("0"),
            String::from("4"),
            String::from("3"),
        ]);

        raw_vec.swap_remove(2);
        assert_equal(raw_vec.as_slice::<String>(), &[
            String::from("0"),
            String::from("4"),
        ]);
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
