Type erased vector. All elements have the same type.

Designed to be type-erased as far as possible - most of the operations does not know about concrete type.

Only destruct operations have additional overhead of indirect call.

# Usage

```rust
    let mut vec: AnyVec = AnyVec::new::<String>();
    vec.push(String::from("0"));
    vec.push(String::from("1"));
    vec.push(String::from("2"));
 
    let mut other_vec: AnyVec = AnyVec::new::<String>();
    // Fully type erased element move from one vec to another
    // without intermediate mem-copies.
    //
    // Equivalent to:
    //
    // let element = vec.swap_remove(0);
    // other.push(element);
    unsafe{
        let element: &mut[u8] = other_vec.push_uninit();    // allocate element 
        vec.swap_take_bytes_into(0, element);               // swap_remove
    }

    // Output 2 1
    for s in vec.as_slice::<String>(){
        println!("{}", s);
    }
    
```

# Known alternatives

* [type_erased_vec](https://crates.io/crates/type_erased_vec). Allow to store `Vec<T>` in type erased way, 
but you need to perform operations, you need to "cast" to concrete type first.
* [untyped_vec](https://crates.io/crates/untyped_vec). Some operations like `len`, `capacity` performed without type
knowledge; but the rest - require concrete type.
