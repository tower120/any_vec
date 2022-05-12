use std::time::{Duration, Instant};
use criterion::{criterion_group, criterion_main, Criterion};
use any_vec::AnyVec;

const SIZE: usize = 10000;

fn vec_swap_remove() -> Duration {
    let mut vec = Vec::new();
    for i in 0..SIZE{
        vec.push([i;1]);
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.swap_remove(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove() -> Duration {
    let mut vec = AnyVec::new::<[usize;1]>();
    for i in 0..SIZE{
        vec.push([i;1]);
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.swap_remove(0);
        }
    start.elapsed()
}


fn bench_custom<F: FnMut() -> Duration>(c: &mut Criterion, id: &str, mut routine: F) {
    c.bench_function(id, move |b|b.iter_custom(|iters|
        (0..iters).map(|_|routine()).sum()
    ));
}

pub fn bench_swap_remove(c: &mut Criterion) {
    bench_custom(c, "Vec swap_remove", vec_swap_remove);
    bench_custom(c, "AnyVec swap_remove", any_vec_swap_remove);
}

criterion_group!(benches, bench_swap_remove);
criterion_main!(benches);