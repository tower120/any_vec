[![crates.io](https://img.shields.io/crates/v/any_vec.svg)](https://crates.io/crates/any_vec)
[![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)](#license)
[![Docs](https://docs.rs/any_vec/badge.svg)](https://docs.rs/any_vec)
[![CI](https://github.com/tower120/any_vec/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/tower120/any_vec/actions/workflows/ci.yml)

Type erased vector. All elements have the same type.

Designed to be type-erased as far as possible - most of the operations does not know about concrete type.

Only destruct operations have additional overhead of indirect call.

# Usage

```rust
    let mut vec: AnyVec = AnyVec::new::<String>();
    {
        // Typed operations.
        let mut vec = vec.downcast_mut::<String>().unwrap();
        vec.push(String::from("0"));
        vec.push(String::from("1"));
        vec.push(String::from("2"));
    }
 
    let mut other_vec: AnyVec = AnyVec::new::<String>();
    // Fully type erased element move from one vec to another
    // without intermediate mem-copies.
    let element = vec.swap_remove(0);
    other_vec.push(element);

    // Output 2 1
    for s in vec.downcast_ref::<String>().unwrap().as_slice(){
        println!("{}", s);
    } 
```

See [documentation](https://docs.rs/any_vec) for more.

## Send, Sync, Clone

You can make `AnyVec` `Send`able, `Sync`able, `Clone`able:

```rust
use any_vec::AnyVec;
use any_vec::traits::*;
let v1: AnyVec<dyn Cloneable + Sync + Send> = AnyVec::new::<String>();
let v2 = v1.clone();
 ```
 This constraints will be applied compiletime to element type:
```rust
// This will fail to compile. 
let v1: AnyVec<dyn Sync + Send> = AnyVec::new::<Rc<usize>>();
```
## LazyClone

 Whenever possible, `any_vec` type erased elements can be lazily cloned:

```rust
 let mut v1: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
 v1.push(AnyValueWrapper::new(String::from("0")));

 let mut v2: AnyVec<dyn Cloneable> = AnyVec::new::<String>();
 let e = v1.swap_remove(0);
 v2.push(e.lazy_clone());
 v2.push(e.lazy_clone());
```

### Changelog

See [CHANGELOG.md](CHANGELOG.md) for version differences.

#### N.B.

*Currently, minimum rust version is 1.61. This is to simplify implementation.  
If you need `any_vec` to work on lower rust version - please feel free to make an issue.*

# Known alternatives

* [type_erased_vec](https://crates.io/crates/type_erased_vec). Allow to store `Vec<T>` in type erased way, 
but you need to perform operations, you need to "cast" to concrete type first.
* [untyped_vec](https://crates.io/crates/untyped_vec). Some operations like `len`, `capacity` performed without type
knowledge; but the rest - require concrete type.
