use std::mem::size_of_val;
use any_vec::AnyVec;
use any_vec::traits::Cloneable;

#[test]
fn exp(){
    let mut v1: AnyVec<dyn Cloneable> = AnyVec::new::<usize>();
    let mut v2: AnyVec<dyn Sync> = AnyVec::new::<usize>();

    let s1 = size_of_val(&v1);
    let s2 = size_of_val(&v2);

    let t = 1;
}
