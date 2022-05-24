use criterion::{criterion_group, criterion_main, Criterion};
use any_vec::AnyVecRaw;

const SIZE: usize = 10000;

fn vec_push(){
    let mut vec = Vec::new();
    for i in 0..SIZE{
        vec.push(i);
    }
}

fn any_vec_push(){
    let mut any_vec = AnyVecRaw::new::<usize>();
    for i in 0..SIZE{
        let mut vec = any_vec.downcast_mut::<usize>().unwrap();
        vec.push(i);
    }
}

fn any_vec_push_unchecked(){
    let mut any_vec = AnyVecRaw::new::<usize>();
    for i in 0..SIZE{
        let mut vec = unsafe{ any_vec.downcast_mut_unchecked::<usize>() };
        vec.push(i);
    }
}

pub fn bench_push(c: &mut Criterion) {
    c.bench_function("Vec push", |b|b.iter(||vec_push()));
    c.bench_function("AnyVec push", |b|b.iter(||any_vec_push()));
    c.bench_function("AnyVec push_unchecked", |b|b.iter(||any_vec_push_unchecked()));
}

criterion_group!(benches, bench_push);
criterion_main!(benches);