use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use any_vec::AnyVec;

const SIZE: usize = 10000;

fn vec_push(){
    let mut vec = Vec::new();
    for i in 0..SIZE{
        vec.push(i);
    }
}

fn any_vec_push(){
    let mut vec = AnyVec::new::<usize>();
    for i in 0..SIZE{
        vec.push(i);
    }
}

fn any_vec_push_unchecked(){
    let mut vec = AnyVec::new::<usize>();
    for i in 0..SIZE{
        unsafe{ vec.push_unchecked(i); }
    }
}

pub fn bench_push(c: &mut Criterion) {
    c.bench_function("Vec push", |b|b.iter(||vec_push()));
    c.bench_function("AnyVec push", |b|b.iter(||any_vec_push()));
    c.bench_function("AnyVec push_unchecked", |b|b.iter(||any_vec_push_unchecked()));
}

criterion_group!(benches, bench_push);
criterion_main!(benches);