# Artful

Artful is an **adaptive radix tree** library for Rust. At a high-level, it's like a [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html). It is based on the implementation of paper, see [The Adaptive Radix Tree: ARTful indexing for Main-Memory Databases](https://db.in.tum.de/~leis/papers/ART.pdf) for more information.

Artful is available on [crates.io](https://crates.io/crates/artful), and API Documentation is available on [docs.rs](https://docs.rs/artful/latest/artful).

## Features

- API similar to a `BTreeMap<K,V>`
- Support SIMD instructions operations

## Using Artful

[Artful is available on crates.io](https://crates.io/crates/artful) The recommended way to use it is to add a line into your Cargo.toml such as:

```rust
[dependencies]
artful = "0.1.1"
```

## Contribution
