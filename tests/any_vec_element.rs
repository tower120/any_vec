use std::any::TypeId;
use itertools::{any, assert_equal};
use any_vec::any_value::{AnyValue, AnyValueCloneable, AnyValueMut, LazyClone};
use any_vec::AnyVec;
use any_vec::traits::Cloneable;

#[test]
fn lazy_clone_test(){
    let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    {
        let mut vec = any_vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
    }

    let mut any_vec_other: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    {
        let e1 = any_vec.swap_remove(1);
        let lz1 = LazyClone::new(&e1);
        let lz2 = LazyClone::new(&lz1).clone();

        any_vec_other.push(LazyClone::new(&e1));
        any_vec_other.push(LazyClone::new(&lz2));
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

#[test]
fn any_vec_get_test(){
    let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    assert_eq!(any_vec.element_typeid(), TypeId::of::<String>());
    {
        let mut vec = any_vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
    }

    let e1_ref = any_vec.get(1);
    assert_eq!(e1_ref.downcast_ref::<String>().unwrap(), &String::from("1"));
    assert_eq!(e1_ref.value_typeid(), TypeId::of::<String>());

    {
        let e1 = e1_ref.lazy_clone();
        let e2 = e1.lazy_clone();
        let e3 = e2.lazy_clone();

        // Consume in reverse order, since lazy clone source must outlive it.
        assert_eq!(e3.downcast::<String>().unwrap(), String::from("1"));
        assert_eq!(e2.downcast::<String>().unwrap(), String::from("1"));
        assert_eq!(e1.downcast::<String>().unwrap(), String::from("1"));
    }

    // Mutability test
    {
        let e = any_vec.downcast_mut::<String>().unwrap().as_mut_slice().get_mut(1).unwrap();
        *e += "A";

        let str = any_vec.get_mut(1).downcast_mut::<String>().unwrap();
        *str += "B";

        assert_eq!(any_vec.get(1).downcast_ref::<String>().unwrap(), &String::from("1AB"));
    }
}

/*#[test]
fn any_vec_push_to_self_test(){
    let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    {
        let mut vec = any_vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
    }

    let e = any_vec.get(1);
    any_vec.push((*e).clone());
}*/
