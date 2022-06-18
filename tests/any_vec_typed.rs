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
fn drain_all_test() {
    let mut any_vec: AnyVec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push(String::from("0"));
    vec.push(String::from("1"));
    vec.push(String::from("2"));
    vec.push(String::from("3"));

    let mut vec2 = Vec::new();
    for e in vec.drain(..){
        vec2.push(e);
    }
    assert_eq!(any_vec.len(), 0);
    assert_equal(vec2, [
        String::from("0"),
        String::from("1"),
        String::from("2"),
        String::from("3"),
    ]);
}

#[test]
fn any_vec_drain_in_the_middle_test() {
    let mut any_vec: AnyVec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push(String::from("0"));
    vec.push(String::from("1"));
    vec.push(String::from("2"));
    vec.push(String::from("3"));
    vec.push(String::from("4"));

    let mut vec2 = Vec::new();
    for e in vec.drain(1..3){
        vec2.push(e);
    }
    assert_equal(vec2, [
        String::from("1"),
        String::from("2"),
    ]);
    assert_equal(any_vec.downcast_ref::<String>().unwrap().as_slice(), &[
        String::from("0"),
        String::from("3"),
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
    let vec3 = vec2.clone();
    assert_eq!(vec1.len(), 0);
    assert_eq!(vec2.len(), 0);
    assert_eq!(vec3.len(), 0);
}

#[test]
fn any_vec_into_iter_test() {
    let mut any_vec: AnyVec = AnyVec::new::<usize>();
    let mut vec = any_vec.downcast_mut::<usize>().unwrap();
    vec.push(1);
    vec.push(10);
    vec.push(100);

    let mut sum = 0;
    for e in vec{
        sum += *e;
    }
    assert_eq!(sum, 111);
}

/*
#[test]
fn any_vec_index_test() {
    let mut any_vec: AnyVec = AnyVec::new::<usize>();
    let mut vec = any_vec.downcast_mut::<usize>().unwrap();
    vec.push(1);
    vec.push(10);
    vec.push(100);

    assert_eq!(vec[1], 10);
    assert_eq!(vec[..], [1, 10, 100]);
    assert_eq!(vec[1..3], [10, 100]);
}
 */