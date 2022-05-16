use any_vec::AnyVec;

#[test]
pub fn downcast_mut_test(){
    let mut any_vec = AnyVec::new::<String>();
    let mut vec = any_vec.downcast_mut::<String>().unwrap();
    vec.push("Hello".into());
    assert_eq!(any_vec.len(), 1);
}

#[test]
pub fn downcast_ref_test(){
    let any_vec = AnyVec::new::<String>();
    let vec = any_vec.downcast_ref::<String>().unwrap();
    assert_eq!(vec.len(), 0);
}

#[test]
pub fn downcast_mut_ref_cast_test(){
    let any_vec = AnyVec::new::<String>();
    let vec = any_vec.downcast_ref::<String>().unwrap();
    assert_eq!(vec.len(), 0);
}