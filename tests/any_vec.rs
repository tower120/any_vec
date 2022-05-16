use std::mem::forget;
use itertools::{assert_equal};
use any_vec::AnyVec;

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
    let mut any_vec = AnyVec::new::<S>();
    let mut vec = any_vec.downcast_mut::<S>().unwrap();
    vec.push(S{i:1});
    vec.push(S{i:2});
    vec.push(S{i:3});

    assert_equal(vec.as_slice().iter().map(|s|s.i), [1, 2, 3]);
}

#[test]
fn it_works() {
    let mut any_vec = AnyVec::new::<String>();

    unsafe{
        let str1 = "Hello".to_string();
        any_vec.push_uninit().copy_from_slice(any_as_u8_slice(&str1));
        forget(str1);

        let str2 = " to ".to_string();
        any_vec.push_uninit().copy_from_slice(any_as_u8_slice(&str2));
        forget(str2);

        let str3 = "world".to_string();
        any_vec.push_uninit().copy_from_slice(any_as_u8_slice(&str3));
        forget(str3);
    }

    assert_equal(
        any_vec.downcast_ref::<String>().unwrap().as_slice(),
        ["Hello", " to ", "world"]
    );
}

#[test]
pub fn push_with_capacity_test(){
    const SIZE: usize = 10000;
    let mut any_vec = AnyVec::with_capacity::<usize>(SIZE);
    let mut vec = any_vec.downcast_mut::<usize>().unwrap();
    for i in 0..SIZE{
        vec.push(i);
    }

    assert_equal(vec.as_slice().iter().copied(), 0..SIZE);
}

#[test]
fn zero_size_type_test() {
    struct Empty{}
    let mut any_vec = AnyVec::new::<Empty>();
    let mut vec = any_vec.downcast_mut::<Empty>().unwrap();
    vec.push(Empty{});
    vec.push(Empty{});
    vec.push(Empty{});

    let mut i = 0;
    for _ in vec.as_mut_slice(){
        i += 1;
    }
    assert_eq!(i, 3);
}

#[test]
fn swap_remove_test() {
    let mut any_vec = AnyVec::new::<String>();
    {
        let mut vec = any_vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
        vec.push(String::from("3"));
        vec.push(String::from("4"));

        let e: String = vec.swap_remove(1);
        assert_eq!(e, String::from("1"));
        assert_equal(vec.as_slice(), &[
            String::from("0"),
            String::from("4"),
            String::from("2"),
            String::from("3"),
        ]);
    }

    any_vec.swap_remove(2);
    assert_equal(any_vec.downcast_ref::<String>().unwrap().as_slice(), &[
        String::from("0"),
        String::from("4"),
        String::from("3"),
    ]);

    any_vec.swap_remove(2);
    assert_equal(any_vec.downcast_ref::<String>().unwrap().as_slice(), &[
        String::from("0"),
        String::from("4"),
    ]);
}

#[test]
fn type_erased_move_test() {
    let mut any_vec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push(String::from("0"));
    vec.push(String::from("1"));
    vec.push(String::from("2"));
    vec.push(String::from("3"));
    vec.push(String::from("4"));

    let mut other_vec = AnyVec::new::<String>();
    unsafe {
        let element = other_vec.push_uninit();
        any_vec.swap_remove_into(2, element);
    }

    assert_equal(other_vec.downcast_ref::<String>().unwrap().as_slice(), &[
        String::from("2"),
    ]);

    assert_equal(any_vec.downcast_ref::<String>().unwrap().as_slice(), &[
        String::from("0"),
        String::from("1"),
        String::from("4"),
        String::from("3"),
    ]);
}
