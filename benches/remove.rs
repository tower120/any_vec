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

fn vec_remove() -> Duration {
    let mut vec = Vec::new();
    for _ in 0..SIZE{
        vec.push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            vec.remove(0);
        }
    start.elapsed()
}

fn any_vec_remove() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.remove(0);
        }
    start.elapsed()
}

fn any_vec_typed_remove() -> Duration {
    let mut any_vec = AnyVec::new::<Element>();
    for _ in 0..SIZE{
        any_vec.downcast_mut::<Element>().unwrap()
            .push(VALUE.clone());
    }

    let start = Instant::now();
        for _ in 0..SIZE{
            any_vec.downcast_mut::<Element>().unwrap()
                .remove(0);
        }
    start.elapsed()
}

fn any_vec_remove_into() -> Duration {
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

                any_vec.remove_into(0, &mut element_bytes[..]);
                element.assume_init();
            }
        }
    start.elapsed()
}

pub fn bench_remove(c: &mut Criterion) {
    bench_custom(c, "Vec remove", vec_remove);
    bench_custom(c, "AnyVec remove", any_vec_remove);
    bench_custom(c, "AnyVec remove_into", any_vec_remove_into);
    bench_custom(c, "AnyVecTyped remove", any_vec_typed_remove);
}

criterion_group!(benches, bench_remove);
criterion_main!(benches);