use criterion::{criterion_group, criterion_main, Criterion};
use any_vec::AnyVec;

const SIZE: usize = 10000;

fn vec_insert_front(){
    let mut vec = Vec::new();
    for i in 0..SIZE{
        vec.insert(0, i);
    }
}

fn any_vec_insert_front(){
    let mut any_vec = AnyVec::new::<usize>();
    for i in 0..SIZE{
        let mut vec = any_vec.downcast_mut::<usize>().unwrap();
        vec.insert(0, i);
    }
}

fn vec_insert_back(){
    let mut vec = Vec::new();
    for i in 0..SIZE{
        vec.insert(i, i);
    }
}

fn any_vec_insert_back(){
    let mut any_vec = AnyVec::new::<usize>();
    for i in 0..SIZE{
        let mut vec = any_vec.downcast_mut::<usize>().unwrap();
        vec.insert(i, i);
    }
}

pub fn bench_push(c: &mut Criterion) {
    c.bench_function("Vec push front", |b|b.iter(||vec_insert_front()));
    c.bench_function("AnyVec push front", |b|b.iter(||any_vec_insert_front()));
    c.bench_function("Vec push back", |b|b.iter(||vec_insert_back()));
    c.bench_function("AnyVec push back", |b|b.iter(||any_vec_insert_back()));

}

criterion_group!(benches, bench_push);
criterion_main!(benches);