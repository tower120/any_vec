use itertools::assert_equal;
use any_vec::AnyVec;

#[test]
fn remove_test() {
    let mut any_vec: AnyVec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push(String::from("0"));
    vec.push(String::from("1"));
    vec.push(String::from("2"));
    vec.push(String::from("3"));
    vec.push(String::from("4"));

    // type erased remove
    vec.remove(2);
    assert_equal(vec.as_slice(), &[
        String::from("0"),
        String::from("1"),
        String::from("3"),
        String::from("4"),
    ]);

    // remove last
    vec.remove(3);
    assert_equal(vec.as_slice(), &[
        String::from("0"),
        String::from("1"),
        String::from("3"),
    ]);

    // remove first
    vec.remove(0);
    assert_equal(vec.as_slice(), &[
        String::from("1"),
        String::from("3"),
    ]);
}

#[test]
fn swap_remove_test() {
    let mut any_vec: AnyVec = AnyVec::new::<String>();
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

    vec.swap_remove(2);
    assert_equal(vec.as_slice(), &[
        String::from("0"),
        String::from("4"),
        String::from("3"),
    ]);

    vec.swap_remove(2);
    assert_equal(vec.as_slice(), &[
        String::from("0"),
        String::from("4"),
    ]);
}

#[test]
pub fn downcast_mut_test(){
    let mut any_vec: AnyVec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push("Hello".into());
    assert_eq!(vec.len(), 1);
}

#[test]
pub fn downcast_ref_test(){
    let mut any_vec: AnyVec = AnyVec::new::<String>();
    any_vec.clear();
    let vec1 = any_vec.downcast_ref::<String>().unwrap();
    let vec2 = any_vec.downcast_ref::<String>().unwrap();
    assert_eq!(vec1.len(), 0);
    assert_eq!(vec2.len(), 0);
}
