mod utils;

use std::mem::{MaybeUninit, size_of};
use std::ptr::{slice_from_raw_parts_mut};
use std::time::{Duration, Instant};
use criterion::{criterion_group, criterion_main, Criterion};
use any_vec::AnyVec;
use crate::utils::bench_custom;

const SIZE: usize = 10000;

type Element = String;
static VALUE: Element = String::new();

fn vec_swap_remove() -> Duration {
    let mut vec = Vec::new();
    for _ in 0..SIZE{
        vec.push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.swap_remove(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.swap_remove(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove_v3() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.swap_remove_v3(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove_v4() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.swap_remove_v4_test(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove_v5() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.swap_remove_v5(0);
        }
    start.elapsed()
}


fn any_vec_typed_swap_remove() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.downcast_mut::<Element>().unwrap()
                .swap_remove(0);
        }
    start.elapsed()
}

fn any_vec_typed_swap_remove_v2() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.downcast_mut::<Element>().unwrap()
                .swap_remove_v2(0);
        }
    start.elapsed()
}

fn any_vec_swap_remove_into() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            unsafe{
                let mut element = MaybeUninit::<Element>::uninit();
                let element_bytes = &mut *slice_from_raw_parts_mut(
                    element.as_mut_ptr() as *mut u8,
                    size_of::<Element>()
                );

                any_vec.swap_remove_into(0, &mut element_bytes[..]);
                element.assume_init();
            }
        }
    start.elapsed()
}

pub fn bench_swap_remove(c: &mut Criterion) {
    bench_custom(c, "Vec swap_remove", vec_swap_remove);
    // bench_custom(c, "AnyVec swap_remove", any_vec_swap_remove);
    // bench_custom(c, "AnyVec swap_remove_v3", any_vec_swap_remove_v3);
    // bench_custom(c, "AnyVec swap_remove_v4", any_vec_swap_remove_v4);
    bench_custom(c, "AnyVec swap_remove_v5", any_vec_swap_remove_v5);
    // bench_custom(c, "AnyVec swap_remove_into", any_vec_swap_remove_into);
//     bench_custom(c, "AnyVecTyped swap_remove", any_vec_typed_swap_remove);
//     bench_custom(c, "AnyVecTyped swap_remove_v2", any_vec_typed_swap_remove_v2);
}

criterion_group!(benches, bench_swap_remove);
criterion_main!(benches);