/*!
[protobuf](https://protobuf.dev/) support for `sval`.

This library implements a binary encoding for `sval::Value`s that's compatible with the
protobuf [wire format](https://protobuf.dev/programming-guides/encoding/).

This library doesn't depend on `protoc` or any code generation tools; any implementation of
`sval::Value` can be encoded. You can use it in cases where standard code generation is either
impractical or produces undesirable results. It supports some more niche use-cases like embedding
already encoded messages into others without needing to parse them first.

This library only supports encoding.

## Specifics

This section uses [protoscope](https://github.com/protocolbuffers/protoscope) syntax for
encoded messages.

### Messages

If you `#[derive(Value)]`, your structs are encoded as messages:

```rust
# use sval_derive::*;
# fn main() {}
# fn _wrapper() -> impl sval::Value {
#[derive(Value)]
pub struct Record<'a> {
    id: i32,
    title: &'a str,
    data: &'a str,
}

Record {
    id: 42,
    title: "My Message",
    data: "Some amazing content",
}
# }
```

```text
1: 42
2: {"My Message"}
3: {"Some amazing content"}
```

Specify an `#[sval(index)]` on fields to set the field number explicitly:

```rust
# use sval_derive::*;
# fn main() {}
# fn _wrapper() -> impl sval::Value {
#[derive(Value)]
pub struct ManuallyIndexed<'a> {
    #[sval(index = 3)]
    id: i32,
    #[sval(index = 7)]
    title: &'a str,
}

ManuallyIndexed {
    id: 42,
    title: "My Message",
}
# }
```

```text
3: 42
7: {"My Message"}
```

Anonymous tuples are also messages:

```rust,no_run
# use sval_derive::*;
# fn main() {}
# fn _wrapper() -> impl sval::Value {
(
    42,
    "My Message",
    "Some amazing content",
)
# }
```

```text
1: 42
2: {"My Message"}
3: {"Some amazing content"}
```

128bit numbers are always encoded as a 16 byte buffer with the little-endian bytes of the value.
*/

//#![no_std]

extern crate alloc;

mod stream;
pub use self::stream::*;

pub mod buf;
pub mod tags;

pub mod raw;
