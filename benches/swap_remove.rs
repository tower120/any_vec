use std::time::{Duration, Instant};
use criterion::{criterion_group, criterion_main, Criterion};
use any_vec::AnyVec;

const SIZE: usize = 10000;

type Element = [usize;1];
static VALUE: Element = [0;1];

fn vec_swap_remove() -> Duration {
    let mut vec = Vec::new();
    for _ in 0..SIZE{
        vec.push(VALUE);
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.swap_remove(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove() -> Duration {
    let mut vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        vec.push(VALUE);
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.swap_remove(0);
        }
    start.elapsed()
}

fn any_vec_swap_take() -> Duration {
    let mut vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        vec.push(VALUE);
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.swap_take::<Element>(0);
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
    bench_custom(c, "AnyVec swap_take", any_vec_swap_take);
}

criterion_group!(benches, bench_swap_remove);
criterion_main!(benches);