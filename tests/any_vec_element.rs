use std::any::TypeId;
use any_vec::any_value::AnyValue;
use any_vec::AnyVec;
use any_vec::traits::Cloneable;

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
        let mut e1 = (*e1_ref).clone();
        let mut e2 = (*e1_ref).clone();
        let mut e3 = (*e1_ref).clone();

        e3.downcast_mut::<String>().unwrap().len();

        assert_eq!(e1.downcast::<String>(), String::from("1"));
    }
}