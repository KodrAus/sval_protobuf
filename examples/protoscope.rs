/*
This example demonstrates how various Rust types are encoded
to protobuf through sval.

You'll need the `protoscope` tool available on the path. You
can grab it via: https://github.com/protocolbuffers/protoscope
*/

use std::fmt;

use sval_derive::*;

#[derive(Debug, Value)]
pub struct Record<'a> {
    id: i32,
    title: &'a str,
    data: &'a str,
}

#[derive(Debug, Value)]
pub struct OneOfRecord<'a> {
    id: i32,
    title: &'a str,
    #[sval(flatten)]
    data: OneOfRecordData<'a>,
}

#[derive(Debug, Value)]
pub enum OneOfRecordData<'a> {
    Bool(bool),
    Text(&'a str),
}

#[derive(Debug, Value)]
pub struct ManuallyIndexed<'a> {
    #[sval(index = 3)]
    id: i32,
    #[sval(index = 7)]
    title: &'a str,
}

#[derive(Debug, Value)]
pub struct Tagged<'a> {
    pre_encoded: &'a sval::BinarySlice,
    #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_LEN_PACKED")]
    packed: &'a [i32],
}

fn main() {
    print_all(&[
        inspect(42),
        inspect(3.14),
        inspect(u128::MAX),
        inspect("Some text"),
        inspect(&[1, 2, 3, 4, 5]),
        inspect((42, "My Message", "Some amazing content")),
        inspect(Record {
            id: 42,
            title: "My Message",
            data: "Some amazing content",
        }),
        inspect(OneOfRecord {
            id: 42,
            title: "My Message",
            data: OneOfRecordData::Text("Some amazing content"),
        }),
        inspect(ManuallyIndexed {
            id: 42,
            title: "My Message",
        }),
        inspect(Tagged {
            pre_encoded: sval::BinarySlice::new(&[
                8, 1, 18, 12, 83, 111, 109, 101, 32, 99, 111, 110, 116, 101, 110, 116,
            ]),
            packed: &[1, 2, 3, 4, 5],
        }),
    ]);
}

fn print_all(items: &[String]) {
    let mut first = true;

    for item in items {
        if !first {
            println!("----\n");
        }

        println!("{item}");

        first = false;
    }
}

fn inspect(value: impl sval::Value + fmt::Debug) -> String {
    let encoded = protoscope(&*sval_protobuf::stream_to_protobuf(&value).to_vec());

    format!("{value:#?}\n\n{encoded}")
}

fn protoscope(encoded: &[u8]) -> String {
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
    };

    let mut protoscope = Command::new("protoscope")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to call protoscope");

    let mut stdin = protoscope.stdin.take().expect("missing stdin");
    stdin.write_all(encoded).expect("failed to write");
    drop(stdin);

    let mut buf = String::new();
    protoscope
        .stdout
        .take()
        .expect("missing stdout")
        .read_to_string(&mut buf)
        .expect("failed to read");

    buf
}
