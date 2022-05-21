use std::{mem, ptr};
use std::alloc::{alloc, dealloc, Layout, realloc, handle_alloc_error};
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::{NonNull, null_mut};
use crate::{AnyVecMut, AnyVecRef, AnyVecTyped, copy_bytes_nonoverlapping, swap_bytes_nonoverlapping};
use crate::any_value::{AnyValueTemp, AnyValue};
use crate::any_value_tmp2::AnyValueTemp2;
use crate::swap_remove::{SwapRemove, SwapRemove2};

/// Type erased vec-like container.
/// All elements have the same type.
///
/// Only destruct operations have indirect call overhead.
///
/// *`Element: 'static` due to TypeId requirements*
pub struct AnyVec {
    pub(crate) mem: NonNull<u8>,
    capacity: usize,        // in elements
    pub(crate) len: usize,             // in elements
    element_layout: Layout, // size is aligned
    type_id: TypeId,        // purely for safety checks
    pub(crate) drop_fn: Option<fn(ptr: *mut u8, len: usize)>
}

impl AnyVec {
    pub fn new<Element: 'static>() -> Self {
        Self::with_capacity::<Element>(0)
    }

    pub fn with_capacity<Element: 'static>(capacity: usize) -> Self {
        let mut this = Self{
            mem: NonNull::<u8>::dangling(),
            capacity: 0,
            len: 0,
            element_layout: Layout::new::<Element>(),
            type_id: TypeId::of::<Element>(),
            drop_fn:
                if !mem::needs_drop::<Element>(){
                    None
                } else{
                    Some(|mut ptr: *mut u8, len: usize|{
                        for _ in 0..len{
                            unsafe{
                                ptr::drop_in_place(ptr as *mut Element);
                                ptr = ptr.add(mem::size_of::<Element>());
                            }
                        }
                    })
                }
        };
        this.set_capacity(capacity);
        this
    }

    #[inline]
    pub fn downcast_ref<Element: 'static>(&self) -> Option<AnyVecRef<Element>> {
        if self.type_id == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<Element: 'static>(&self) -> AnyVecRef<Element> {
        AnyVecRef{
            any_vec_typed: (AnyVecTyped::new(
                NonNull::new_unchecked(self as *const _ as *mut _)
            ))
        }
    }

    #[inline]
    pub fn downcast_mut<Element: 'static>(&mut self) -> Option<AnyVecMut<Element>> {
        if self.type_id == TypeId::of::<Element>() {
            unsafe{ Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<Element: 'static>(&mut self) -> AnyVecMut<Element> {
        AnyVecMut{
            any_vec_typed: AnyVecTyped::new(NonNull::new_unchecked(self))
        }
    }

    /// This is the only function, which do allocations/deallocations.
    /// Real capacity one element bigger. Last virtual element used by remove operations,
    /// as temporary value location.
    fn set_capacity(&mut self, new_capacity: usize){
        // Never cut
        debug_assert!(self.len <= new_capacity);

        if self.capacity == new_capacity {
            return;
        }

        if self.element_layout.size() != 0 {
            unsafe{
                // Non checked mul, because this memory size already allocated.
                // +1 - from temporary element storage
                let mem_layout = Layout::from_size_align_unchecked(
                    self.element_layout.size() * (self.capacity + 1),
                    self.element_layout.align()
                );

                self.mem =
                    if new_capacity == 0 {
                        dealloc(self.mem.as_ptr(), mem_layout);
                        NonNull::<u8>::dangling()
                    } else {
                        // mul carefully, to prevent overflow.
                        // +1 - for temporary element storage
                        let new_mem_size = self.element_layout.size()
                            .checked_mul(new_capacity + 1).unwrap();
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

    #[inline]
    fn index_check(&self, index: usize){
        assert!(index < self.len, "Index out of range!");
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self){
        self.set_capacity(
             if self.capacity == 0 {2} else {self.capacity * 2}
        );
    }

    /// Inserts one element without actually writing anything at position index,
    /// shifting all elements after it to the right.
    ///
    /// Return byte slice, that must be filled with element data.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    ///
    /// # Safety
    /// * returned byte slice must be written with actual Element bytes.
    /// * Element bytes must be aligned.
    /// * Element must be "forgotten".
    pub unsafe fn insert_uninit(&mut self, index: usize) -> &mut[u8] {
        assert!(index <= self.len, "Index out of range!");
        if self.len == self.capacity{
            self.grow();
        }

        let element = self.mem.as_ptr().add(self.element_layout.size() * index);
        let next_element = element.add(self.element_layout.size());

        // push right
        ptr::copy(
            element,
            next_element,
            self.element_layout.size() * (self.len - index)
        );
        self.len += 1;

        std::slice::from_raw_parts_mut(
            element,
            self.element_layout.size(),
        )
    }

    /// Pushes one element without actually writing anything.
    ///
    /// Return byte slice, that must be filled with element data.
    ///
    /// # Safety
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

    /// # Panics
    ///
    /// * Panics if type mismatch.
    /// * Panics if out of memory.
    #[inline]
    pub fn push<V: AnyValue>(&mut self, value: V) {
        assert_eq!(value.value_typeid(), self.type_id);
        if self.len == self.capacity{
            self.grow();
        }

        // Compile time optimization
        let element_size = V::KNOWN_SIZE.unwrap_or(self.element_layout.size());

        unsafe{
        let new_element = self.mem.as_ptr().add(element_size * self.len);
        value.consume_bytes(|value_bytes|{
            copy_bytes_nonoverlapping(
                value_bytes.as_ptr(),
                new_element,
                element_size
            );
        });
        }

        self.len += 1;
    }

    // TODO: hide
    #[inline]
    pub(crate) fn drop_element(&mut self, ptr: *mut u8, len: usize){
        if let Some(drop_fn) = self.drop_fn{
            (drop_fn)(ptr, len);
        }
    }

    /// element_size as parameter - because it possible can be known at compile time
    #[inline]
    pub(crate) unsafe fn remove_into_impl(&mut self, index: usize, element_size: usize, out: *mut u8) {
        self.index_check(index);

        let element = self.mem.as_ptr().add(element_size * index);

        // 1. copy element to out
        if !out.is_null() {
            copy_bytes_nonoverlapping(element, out, element_size);
        }

        // 2. shift everything left
        ptr::copy(
            element.add(element_size),
            element,
            element_size * (self.len - index - 1)
        );

        // 3. shrink len
        self.len -= 1;
    }

    /// Type erased version of [`Vec::remove`]. Due to this, does not return element.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn remove(&mut self, index: usize) {
    unsafe{
        let temp_element = if self.drop_fn.is_some() {
            self.mem.as_ptr().add(self.element_layout.size() * self.capacity)
        } else {
            null_mut()
        };

        self.remove_into_impl(index, self.element_layout.size(), temp_element);

        // drop element in temporary storage
        if !temp_element.is_null() {
            (self.drop_fn.unwrap_unchecked())(temp_element, 1);
        }
    }
    }

    /// Same as [`remove`], but copy removed element as bytes to `out`.
    ///
    /// [`remove`]: Self::remove
    ///
    /// # Safety
    /// * It is your responsibility to properly drop `out` element.
    ///
    /// # Panics
    /// * Panics if index out of bounds.
    /// * Panics if out len does not match element size.
    #[inline]
    pub unsafe fn remove_into(&mut self, index: usize, out: &mut[u8]) {
        assert_eq!(out.len(), self.element_layout.size());
        self.remove_into_impl(index, self.element_layout.size(), out.as_mut_ptr());
    }

    /// Type erased version of [`Vec::swap_remove`]. Due to this, does not return element.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) {
    unsafe{
        self.index_check(index);

        let element = self.mem.as_ptr().add(self.element_layout.size() * index);

        // 1. swap elements
        let last_index = self.len - 1;
        let last_element = self.mem.as_ptr().add(self.element_layout.size() * last_index);
        if index != last_index {
            if self.drop_fn.is_none(){
                copy_bytes_nonoverlapping(last_element, element, self.element_layout.size());
            } else {
                swap_bytes_nonoverlapping(last_element, element, self.element_layout.size());
            }
        }

        // 2. shrink len
        self.len -= 1;

        // 3. drop last
        self.drop_element(last_element, 1);
    }
    }

    // Significantly slower.... Maybe due to additional memory access at temp area?
    // Hide for now.
    #[allow(dead_code)]
    #[inline]
    fn swap_remove_v2(&mut self, index: usize) {
    unsafe{
        self.index_check(index);

        let element = self.mem.as_ptr().add(self.element_layout.size() * index);

        // 1. swap elements
        let last_index = self.len - 1;
        let last_element = self.mem.as_ptr().add(self.element_layout.size() * last_index);
        let mut element_do_drop = last_element;
        if index != last_index {
            if self.drop_fn.is_some(){
                let temp_element = self.mem.as_ptr().add(self.element_layout.size() * self.capacity);
                copy_bytes_nonoverlapping(element, temp_element, self.element_layout.size());
                element_do_drop = temp_element;
            }
            copy_bytes_nonoverlapping(last_element, element, self.element_layout.size());
        }

        // 2. shrink len
        self.len -= 1;

        // 3. drop last
        self.drop_element(element_do_drop, 1);
    }
    }


    #[inline]
    pub fn swap_remove_v4_test(&mut self, index: usize) -> SwapRemove {
        self.index_check(index);

        SwapRemove{
            index,
            any_vec: self,
            phantom: PhantomData
        }
    }

    #[inline]
    pub fn swap_remove_v5(&mut self, index: usize) -> AnyValueTemp2<SwapRemove2> {
        self.index_check(index);

        AnyValueTemp2(SwapRemove2{
            any_vec: self,
            index,
            phantom: PhantomData
        })
    }


    #[inline]
    pub(crate) fn swap_remove_v3_impl(&mut self, element_size: usize, index: usize) -> impl AnyValue + '_{
    unsafe{
        self.index_check(index);

        let element = self.mem.as_ptr().add(element_size * index);
        let typeid = self.type_id;

        let f =
            move |mut element: *mut u8| {
                // if element.is_null(){
                //     element = self.mem.as_ptr().add(element_size * index);
                // } else {
                    // 1. drop
                    self.drop_element(element, 1);
                // }

                // 2. overwrite with last element
                let last_index = self.len - 1;
                let last_element = self.mem.as_ptr().add(element_size * last_index);
                if index != last_index {
                    //copy_bytes_nonoverlapping
                    ptr::copy_nonoverlapping
                        (last_element, element, element_size);
                }

                // 3. shrink len
                self.len -= 1;
            };

        AnyValueTemp::from_raw_parts(NonNull::new_unchecked(element), typeid, f)
    }
    }

    #[inline]
    pub fn swap_remove_v3(&mut self, index: usize) -> impl AnyValue + '_{
        self.swap_remove_v3_impl(self.element_layout.size(), index)
/*    unsafe{
        self.index_check(index);

        let element = self.mem.as_ptr().add(self.element_layout.size() * index);
        let typeid = self.type_id;

        let f =
            move |mut element: *mut u8| {
                if element.is_null(){
                    element = self.mem.as_ptr().add(self.element_layout.size() * index);
                } else {
                    // 1. drop
                    self.drop_element(element, 1);
                }

                // 2. overwrite with last element
                let last_index = self.len - 1;
                let last_element = self.mem.as_ptr().add(self.element_layout.size() * last_index);
                if index != last_index {
                    copy_bytes_nonoverlapping(last_element, element, self.element_layout.size());
                }

                // 3. shrink len
                self.len -= 1;
            };

        AnyValueTemp::from_raw_parts(NonNull::new_unchecked(element), typeid, f)
    }*/
    }

    /// drop element, if out is null.
    /// element_size as parameter - because it possible can be known at compile time
    #[inline]
    pub(crate) unsafe fn swap_remove_into_impl(&mut self, index: usize, element_size: usize, out: *mut u8)
    {
        self.index_check(index);

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
    pub unsafe fn swap_remove_into(&mut self, index: usize, out: &mut[u8]){
        assert_eq!(out.len(), self.element_layout.size());  // This allows compile time optimization!
        self.swap_remove_into_impl(index, self.element_layout.size(), out.as_mut_ptr());
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
    pub(crate) unsafe fn as_slice_unchecked<T>(&self) -> &[T]{
        std::slice::from_raw_parts(
            self.mem.as_ptr().cast::<T>(),
            self.len,
        )
    }

    #[inline]
    pub(crate) unsafe fn as_mut_slice_unchecked<T>(&mut self) -> &mut[T]{
        std::slice::from_raw_parts_mut(
            self.mem.as_ptr().cast::<T>(),
            self.len,
        )
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
