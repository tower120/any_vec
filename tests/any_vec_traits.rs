use itertools::assert_equal;
use any_vec::AnyVec;
use any_vec::traits::*;

#[test]
pub fn test_default(){
    let _any_vec: AnyVec = AnyVec::new::<String>();
    // should not compile
    //fn t(_: impl Sync){}
    //t(any_vec);
}

#[test]
pub fn test_sync(){
    let any_vec: AnyVec<dyn Cloneable + Sync + Send> = AnyVec::new::<String>();
    fn t(_: impl Sync){}
    t(any_vec);
}

#[test]
pub fn test_clone(){
    let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    {
        let mut vec = any_vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
    }

    let any_vec2 = any_vec.clone();
    assert_equal(
        any_vec.downcast_ref::<String>().unwrap().as_slice(),
        any_vec2.downcast_ref::<String>().unwrap().as_slice()
    );
}

#[test]
pub fn type_check_test(){
    let mut any_vec: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
    any_vec.check::<String>();

    let mut any_vec: AnyVec<dyn Send> = AnyVec::new::<String>();
    any_vec.check::<String>();

    let mut any_vec: AnyVec<dyn Sync> = AnyVec::new::<String>();
    any_vec.check::<String>();

    let mut any_vec: AnyVec<dyn Sync + Send> = AnyVec::new::<String>();
    any_vec.check::<String>();

    let mut any_vec: AnyVec<dyn Sync + Send + Cloneable> = AnyVec::new::<String>();
    any_vec.check::<String>();
}

