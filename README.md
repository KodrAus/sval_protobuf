# `sval_protobuf`

[![Rust](https://github.com/KodrAus/sval_protobuf/workflows/protobuf/badge.svg)](https://github.com/KodrAus/sval_protobuf/actions)
[![Latest version](https://img.shields.io/crates/v/sval_protobuf.svg)](https://crates.io/crates/sval_protobuf)
[![Documentation Latest](https://docs.rs/sval_protobuf/badge.svg)](https://docs.rs/sval_protobuf)

[protobuf](https://protobuf.dev/) support for [`sval`](https://docs.rs/sval/latest/sval/).

This library implements a binary encoding for `sval::Value`s that's compatible with the
protobuf [wire format](https://protobuf.dev/programming-guides/encoding/).

It doesn't require `protoc`.

## Getting started

Add `sval_protobuf` and `sval` to your `Cargo.toml`:

```toml
[dependencies.sval]
version = "2"

[dependencies.sval_derive]
version = "2"

[dependencies.sval_protobuf]
version = "0.2.0"
```

Derive `sval::Value` on your types and encode them as protobuf messages:

```rust
#[macro_use]
extern crate sval_derive;

#[derive(Value)]
pub struct Record<'a> {
    id: i32,
    title: &'a str,
    data: &'a str,
}

let encoded = sval_protobuf::stream_to_protobuf(Record {
    id: 42,
    title: "My Message",
    data: "Some extra contents",
});
```
