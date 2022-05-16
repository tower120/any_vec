use std::ops::{Deref, DerefMut};
use any_vec::{AnyVec, AnyVecRef, AnyVecTyped};

#[test]
pub fn downcast_mut_test(){
    let mut any_vec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push("Hello".into());
    assert_eq!(vec.len(), 1);
}

#[test]
pub fn downcast_ref_test(){
    let mut any_vec = AnyVec::new::<String>();
    let vec1 = any_vec.downcast_ref::<String>().unwrap();
    let vec2 = any_vec.downcast_ref::<String>().unwrap();
    //any_vec.clear();
    assert_eq!(vec1.len(), 0);
    assert_eq!(vec2.len(), 0);
}

/*#[test]
pub fn downcast_mut_ref_cast_test(){
    let mut any_vec = AnyVec::new::<String>();
    let vec_mut = any_vec.downcast_mut::<String>().unwrap();
    // let vec_ref1: AnyVecRef<_> = vec_mut.into();
    // let vec_ref2: AnyVecRef<_> = vec_ref1;
    // let vec_ref3: AnyVecRef<_> = vec_ref1;
    // assert_eq!(vec_ref1.len(), 0);
    // assert_eq!(vec_ref2.len(), 0);
    // assert_eq!(vec_ref3.len(), 0);
}*/