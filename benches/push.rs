use std::any::TypeId;
use std::mem::size_of;
use std::ptr::NonNull;
use criterion::{criterion_group, criterion_main, Criterion};
use any_vec::any_value::{AnyValueRawPtr, AnyValueRawTyped};
use any_vec::AnyVec;

const SIZE: usize = 10000;

type Element = (usize, usize);
static VALUE: Element = (0,0);

fn vec_push(){
    let mut vec = Vec::new();
    for _ in 0..SIZE{
        vec.push(VALUE.clone());
    }
}

fn any_vec_push(){
    let mut any_vec: AnyVec = AnyVec::new::<Element>();
    let mut vec = any_vec.downcast_mut::<Element>().unwrap();
    for _ in 0..SIZE{
        vec.push(VALUE.clone());
    }
}

fn any_vec_push_untyped(){
    let mut any_vec: AnyVec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        let value = VALUE.clone();
        let raw_value = unsafe{
            AnyValueRawTyped::new(
            NonNull::from(&value).cast::<u8>(),
            size_of::<Element>(),
            TypeId::of::<Element>()
        )};
        any_vec.push(raw_value);
    }
}

fn any_vec_push_untyped_unchecked(){
    let mut any_vec: AnyVec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        let value = VALUE.clone();
        let raw_value = unsafe{
            AnyValueRawPtr::new(
            NonNull::from(&value).cast::<u8>(),
        )};
        unsafe{
            any_vec.push_unchecked(raw_value);
        }
    }
}

pub fn bench_push(c: &mut Criterion) {
    c.bench_function("Vec push", |b|b.iter(||vec_push()));
    c.bench_function("AnyVec push", |b|b.iter(||any_vec_push()));
    c.bench_function("AnyVec push untyped unchecked", |b|b.iter(||any_vec_push_untyped_unchecked()));
    c.bench_function("AnyVec push untyped", |b|b.iter(||any_vec_push_untyped()));
}

criterion_group!(benches, bench_push);
criterion_main!(benches);