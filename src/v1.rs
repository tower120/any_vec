use std::any::TypeId;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub struct AnyVec{
    raw_vec: RawVec,
    type_id: TypeId,
    drop_fn: fn(&mut RawVec)
}

struct RawVec{
    ptr: *mut u8,
    length: usize,
    capacity: usize
}

impl AnyVec{
    #[inline]
    pub fn new<T: 'static>() -> Self {
        // Prevent running `v`'s destructor so we are in complete control
        // of the allocation.
        let mut v = mem::ManuallyDrop::new(Vec::<T>::new());

        // Pull out the various important pieces of information about `v`
        let raw_vec = RawVec{
            ptr:  v.as_mut_ptr() as *mut u8,
            length: v.len(),
            capacity: v.capacity()
        };

        Self{
            raw_vec,
            type_id: TypeId::of::<T>(),
            drop_fn: |raw_vec: &mut RawVec|{
                let vec = unsafe{
                    Vec::from_raw_parts(
                        raw_vec.ptr as *mut T,
                        raw_vec.length,
                        raw_vec.capacity)
                };
                std::mem::drop(vec);
            }
        }
    }

    /// element should be properly aligned!
    #[inline]
    pub unsafe fn push_raw(&mut self, raw_element: &[u8]){
        let mut vec =
            Vec::from_raw_parts(
                self.raw_vec.ptr,
                self.raw_vec.length,
                self.raw_vec.capacity);
        vec.extend_from_slice(raw_element);
    }

    #[inline]
    pub unsafe fn get_mut_unchecked<T>(&mut self) -> VecMutRef<T>{
        let vec =
            Vec::from_raw_parts(
                self.raw_vec.ptr as *mut T,
                self.raw_vec.length,
                self.raw_vec.capacity);

        VecMutRef{
            vec: ManuallyDrop::new(vec),
            phantom: PhantomData
        }
    }

    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> VecMutRef<T>{
        assert_eq!(TypeId::of::<T>(), self.type_id);
        unsafe{ self.get_mut_unchecked::<T>() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw_vec.length
    }
}

impl Drop for AnyVec{
    #[inline]
    fn drop(&mut self) {
        (self.drop_fn)(&mut self.raw_vec);
    }
}

pub struct VecMutRef<'a, T>{
    vec: ManuallyDrop<Vec<T>>,
    phantom: PhantomData<&'a mut T>
}
impl<'a, T> Deref for VecMutRef<'a, T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        self.vec.deref()
    }
}
impl<'a, T> DerefMut for VecMutRef<'a, T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vec.deref_mut()
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;
    use crate::v1::AnyVec;

    #[test]
    fn it_works() {
        let mut any_vec = AnyVec::new::<String>();
        let mut vec = any_vec.get_mut::<String>();
        vec.push("Hello".to_string());
        vec.push(" ".to_string());
        vec.push("world".to_string());

        assert_equal(vec.iter(), ["Hello", " ", "world"]);
    }
}
