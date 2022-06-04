// TODO: THIS IS TEMPORARY!!!

use itertools::assert_equal;
use any_vec::any_value::LazyClone;
use any_vec::AnyVec;
use any_vec::traits::Cloneable;

#[test]
fn test(){
    let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    {
        let mut vec = any_vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
    }

    let mut any_vec_other: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    {
        let e1 = any_vec.swap_remove2(1);
        let lz1 = LazyClone::new(&e1);
        let lz2 = LazyClone::new(&lz1).clone();

        any_vec_other.push2(LazyClone::new(&e1));
        any_vec_other.push2(LazyClone::new(&lz2));
    }

    assert_equal(
        any_vec.downcast_ref::<String>().unwrap().as_slice(),
        &[String::from("0"), String::from("2")]
    );
    assert_equal(
        any_vec_other.downcast_ref::<String>().unwrap().as_slice(),
        &[String::from("1"), String::from("1")]
    );
}