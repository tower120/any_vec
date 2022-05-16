use std::mem::forget;
use itertools::{any, assert_equal};
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

#[test]
fn vec_mut_test() {
    let mut any_vec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    //any_vec.push(String::from("0"));
    //vec.push("Hello".into());
    //any_vec.push(String::from("0"));
    //any_vec.len();
    //vec.push("Hello".into());
    //any_vec.len();
}