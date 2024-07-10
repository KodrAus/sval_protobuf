pub mod protos {
    pub mod cases {
        include!(concat!(env!("OUT_DIR"), "/sval.protobuf.cases.rs"));
    }

    pub mod opentelemetry {
        pub mod proto {
            pub mod collector {
                pub mod logs {
                    pub mod v1 {
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/opentelemetry.proto.collector.logs.v1.rs"
                        ));
                    }
                }
            }

            pub mod common {
                pub mod v1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.common.v1.rs"
                    ));
                }
            }

            pub mod logs {
                pub mod v1 {
                    include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.logs.v1.rs"));
                }
            }

            pub mod resource {
                pub mod v1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.resource.v1.rs"
                    ));
                }
            }
        }
    }
}

pub mod opentelemetry;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use prost::Message;
    use sval_derive::*;
    use sval_protobuf::buf::{ProtoBuf, ProtoBufMut};

    #[test]
    fn basic() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::Basic {
                id: 1,
                content: "Some content".to_owned(),
                index: None,
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_varint(1);
            buf.push_varint_uint64(1);

            buf.push_field_len(2);
            buf.begin_len(());
            buf.push(b"Some content");
            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval1 = sval_protobuf::stream_to_protobuf((1, "Some content", None::<i32>))
            .to_vec()
            .into_owned();

        let sval2 = {
            #[derive(Value)]
            pub struct Basic<'a> {
                id: i32,
                content: &'a str,
                index: Option<i32>,
            }

            sval_protobuf::stream_to_protobuf(Basic {
                id: 1,
                content: "Some content",
                index: None,
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[test]
    fn basic_scalar() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicScalar {
                f64: 3.1415,
                f32: 3.14,
                vi32: i32::MIN,
                vi64: i64::MIN,
                vu32: u32::MAX,
                vu64: u64::MAX,
                si32: i32::MIN,
                si64: i64::MIN,
                fi32: u32::MAX,
                fi64: u64::MAX,
                sfi32: i32::MIN,
                sfi64: i64::MIN,
                bool: true,
                sbin: "abc".to_string(),
                bin: b"123".to_vec(),
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_i64(1);
            buf.push_i64_double(3.1415);

            buf.push_field_i32(2);
            buf.push_i32_float(3.14);

            buf.push_field_varint(3);
            buf.push_varint_sint64(i32::MIN as i64);

            buf.push_field_varint(4);
            buf.push_varint_sint64(i64::MIN);

            buf.push_field_varint(5);
            buf.push_varint_uint64(u32::MAX as u64);

            buf.push_field_varint(6);
            buf.push_varint_uint64(u64::MAX);

            buf.push_field_varint(7);
            buf.push_varint_sint64z(i32::MIN as i64);

            buf.push_field_varint(8);
            buf.push_varint_sint64z(i64::MIN);

            buf.push_field_i32(9);
            buf.push_i32_fixed32(u32::MAX);

            buf.push_field_i64(10);
            buf.push_i64_fixed64(u64::MAX);

            buf.push_field_i32(11);
            buf.push_i32_sfixed32(i32::MIN);

            buf.push_field_i64(12);
            buf.push_i64_sfixed64(i64::MIN);

            buf.push_field_varint(13);
            buf.push_varint_bool(true);

            buf.push_field_len(14);
            buf.push_len_varint_uint64(3);
            buf.push(b"abc");

            buf.push_field_len(15);
            buf.push_len_varint_uint64(3);
            buf.push(b"123");

            buf.freeze().to_vec().into_owned()
        };

        let (sval1, sval2) = {
            #[derive(Value)]
            pub struct BasicScalar<'a> {
                f64: f64,
                f32: f32,
                vi32: i32,
                vi64: i64,
                vu32: u32,
                vu64: u64,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_VARINT_SIGNED")]
                si32: i32,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_VARINT_SIGNED")]
                si64: i64,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I32")]
                fi32: u32,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I64")]
                fi64: u64,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I32")]
                sfi32: i32,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I64")]
                sfi64: i64,
                bool: bool,
                sbin: &'a str,
                bin: &'a sval::BinarySlice,
            }

            let buf = sval_protobuf::stream_to_protobuf(BasicScalar {
                f64: 3.1415,
                f32: 3.14,
                vi32: i32::MIN,
                vi64: i64::MIN,
                vu32: u32::MAX,
                vu64: u64::MAX,
                si32: i32::MIN,
                si64: i64::MIN,
                fi32: u32::MAX,
                fi64: u64::MAX,
                sfi32: i32::MIN,
                sfi64: i64::MIN,
                bool: true,
                sbin: "abc",
                bin: sval::BinarySlice::new(b"123"),
            });

            let sval1 = buf.to_vec().into_owned();

            let mut sval2 = Vec::new();
            buf.into_cursor().copy_to_vec(&mut sval2);

            (sval1, sval2)
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[test]
    fn basic_scalar_default() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicScalar {
                f64: 0.0,
                f32: 0.0,
                vi32: 0,
                vi64: 0,
                vu32: 0,
                vu64: 0,
                si32: 0,
                si64: 0,
                fi32: 0,
                fi64: 0,
                sfi32: 0,
                sfi64: 0,
                bool: false,
                sbin: "".to_string(),
                bin: b"".to_vec(),
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let sval = {
            #[derive(Value)]
            pub struct BasicScalar<'a> {
                f64: f64,
                f32: f32,
                vi32: i32,
                vi64: i64,
                vu32: u32,
                vu64: u64,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_VARINT_SIGNED")]
                si32: i32,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_VARINT_SIGNED")]
                si64: i64,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I32")]
                fi32: i32,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I64")]
                fi64: i64,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I32")]
                sfi32: i32,
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_I64")]
                sfi64: i64,
                bool: bool,
                sbin: &'a str,
                bin: &'a sval::BinarySlice,
            }

            sval_protobuf::stream_to_protobuf(BasicScalar {
                f64: 0.0,
                f32: 0.0,
                vi32: 0,
                vi64: 0,
                vu32: 0,
                vu64: 0,
                si32: 0,
                si64: 0,
                fi32: 0,
                fi64: 0,
                sfi32: 0,
                sfi64: 0,
                bool: false,
                sbin: "",
                bin: sval::BinarySlice::new(b""),
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &sval);
    }

    #[test]
    fn basic_non_contiguous_fields() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicNonContiguousFields {
                id: 1,
                content: "Some content".to_owned(),
                index: Some(8),
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_varint(4);
            buf.push_varint_uint64(1);

            buf.push_field_len(11);
            buf.begin_len(());
            buf.push(b"Some content");
            buf.end_len();

            buf.push_field_varint(19);
            buf.push_varint_uint64(8);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            pub struct BasicNonContiguousFields<'a> {
                #[sval(index = 4)]
                id: i32,
                #[sval(index = 11)]
                content: &'a str,
                #[sval(index = 19)]
                index: Option<i32>,
            }

            sval_protobuf::stream_to_protobuf(BasicNonContiguousFields {
                id: 1,
                content: "Some content",
                index: Some(8),
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval);
    }

    #[test]
    fn basic_optional() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicOptional { a: Some(1) }
                .encode(&mut buf)
                .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_varint(1);
            buf.push_varint_uint64(1);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            pub struct BasicOptional {
                a: Option<i32>,
            }

            sval_protobuf::stream_to_protobuf(BasicOptional { a: Some(1) })
                .to_vec()
                .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval);
    }

    #[test]
    fn basic_repeated() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicRepeated {
                a: vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(3);
            buf.begin_len(());
            buf.push(b"1");
            buf.end_len();

            buf.push_field_len(3);
            buf.begin_len(());
            buf.push(b"2");
            buf.end_len();

            buf.push_field_len(3);
            buf.begin_len(());
            buf.push(b"3");
            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval1 = {
            #[derive(Value)]
            pub struct BasicRepeated<'a> {
                #[sval(index = 3)]
                a: &'a [&'a str],
            }

            sval_protobuf::stream_to_protobuf(BasicRepeated {
                a: &["1", "2", "3"],
            })
            .to_vec()
            .into_owned()
        };

        let sval2 = {
            #[derive(Value)]
            pub struct BasicRepeated<'a> {
                #[sval(index = 3)]
                a: &'a [Option<&'a str>],
            }

            sval_protobuf::stream_to_protobuf(BasicRepeated {
                a: &[None, Some("1"), None, Some("2"), Some("3"), None, None],
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[test]
    fn basic_repeated_packed() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicRepeatedPacked { a: vec![1, 2, 3] }
                .encode(&mut buf)
                .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());
            buf.push_varint_uint64(1);
            buf.push_varint_uint64(2);
            buf.push_varint_uint64(3);
            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            pub struct BasicRepeated<'a> {
                #[sval(data_tag = "sval_protobuf::tags::PROTOBUF_LEN_PACKED")]
                a: &'a [i32],
            }

            sval_protobuf::stream_to_protobuf(BasicRepeated { a: &[1, 2, 3] })
                .to_vec()
                .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval);
    }

    #[test]
    fn basic_map() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicMap {
                a: {
                    let mut map = BTreeMap::new();
                    map.insert("a".to_owned(), 1);
                    map.insert("b".to_owned(), 2);
                    map.insert("c".to_owned(), 3);
                    map
                },
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());
            buf.push_field_len(1);
            buf.begin_len(());
            buf.push(b"a");
            buf.end_len();
            buf.push_field_varint(2);
            buf.push_varint_uint64(1);
            buf.end_len();

            buf.push_field_len(1);
            buf.begin_len(());
            buf.push_field_len(1);
            buf.begin_len(());
            buf.push(b"b");
            buf.end_len();
            buf.push_field_varint(2);
            buf.push_varint_uint64(2);
            buf.end_len();

            buf.push_field_len(1);
            buf.begin_len(());
            buf.push_field_len(1);
            buf.begin_len(());
            buf.push(b"c");
            buf.end_len();
            buf.push_field_varint(2);
            buf.push_varint_uint64(3);
            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            pub struct BasicMap<'a> {
                a: &'a sval::MapSlice<&'a str, i32>,
            }

            sval_protobuf::stream_to_protobuf(BasicMap {
                a: sval::MapSlice::new(&[("a", 1), ("b", 2), ("c", 3)]),
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval);
    }

    #[test]
    fn basic_enum() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicEnum {
                value: protos::cases::Enum::B as i32,
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_varint(1);
            buf.push_varint_sint64(-3);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            #[repr(i32)]
            #[allow(dead_code)]
            pub enum Enum {
                A = -1,
                B = -3,
                C = -6,
            }

            #[derive(Value)]
            pub struct BasicEnum {
                value: Enum,
            }

            sval_protobuf::stream_to_protobuf(BasicEnum { value: Enum::B })
                .to_vec()
                .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval);
    }

    #[test]
    fn basic_oneof() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::BasicOneof {
                value: Some(protos::cases::basic_oneof::Value::Boolean(true)),
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_varint(2);
            buf.push_varint_bool(true);

            buf.freeze().to_vec().into_owned()
        };

        let sval1 = {
            #[derive(Value)]
            #[allow(dead_code)]
            pub enum Value<'a> {
                Number(i32),
                Boolean(bool),
                Text(&'a str),
            }

            #[derive(Value)]
            pub struct BasicOneof<'a> {
                #[sval(flatten)]
                value: Value<'a>,
            }

            sval_protobuf::stream_to_protobuf(BasicOneof {
                value: Value::Boolean(true),
            })
            .to_vec()
            .into_owned()
        };

        let sval2 = {
            #[derive(Value)]
            #[allow(dead_code)]
            pub enum Value<'a> {
                Number(i32),
                Boolean(bool),
                Text(&'a str),
            }

            sval_protobuf::stream_to_protobuf(Value::Boolean(true))
                .to_vec()
                .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[test]
    fn nested_oneof() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::NestedOneof {
                a: Some(protos::cases::BasicOneof {
                    value: Some(protos::cases::basic_oneof::Value::Boolean(true)),
                }),
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());
            buf.push_field_varint(2);
            buf.push_varint_bool(true);
            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval1 = {
            #[derive(Value)]
            #[allow(dead_code)]
            pub enum Value<'a> {
                Number(i32),
                Boolean(bool),
                Text(&'a str),
            }

            #[derive(Value)]
            pub struct BasicOneof<'a> {
                #[sval(flatten)]
                value: Value<'a>,
            }

            #[derive(Value)]
            pub struct NestedOneof<'a> {
                a: BasicOneof<'a>,
            }

            sval_protobuf::stream_to_protobuf(NestedOneof {
                a: BasicOneof {
                    value: Value::Boolean(true),
                },
            })
            .to_vec()
            .into_owned()
        };

        let sval2 = {
            #[derive(Value)]
            #[allow(dead_code)]
            pub enum Value<'a> {
                Number(i32),
                Boolean(bool),
                Text(&'a str),
            }

            #[derive(Value)]
            pub struct NestedOneof<'a> {
                value: Value<'a>,
            }

            sval_protobuf::stream_to_protobuf(NestedOneof {
                value: Value::Boolean(true),
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[test]
    fn nested() {
        let prost = {
            let mut buf = Vec::new();

            protos::cases::Nested {
                a: Some(protos::cases::NestedInner {
                    a: Some(protos::cases::BasicOptional { a: Some(1) }),
                    b: b"Some bytes".to_vec(),
                    c: 2,
                }),
                b: "Some text".to_owned(),
                c: 2,
            }
            .encode(&mut buf)
            .unwrap();

            buf
        };

        let raw = {
            let mut buf = ProtoBufMut::new(());

            // a: NestedInner
            buf.push_field_len(1);
            buf.begin_len(());

            // a: Basic
            buf.push_field_len(1);
            buf.begin_len(());
            buf.push_field_varint(1);
            buf.push_varint_uint64(1);
            buf.end_len();

            // b: bytes
            buf.push_field_len(2);
            buf.begin_len(());
            buf.push(b"Some bytes");
            buf.end_len();

            // c: int64
            buf.push_field_varint(3);
            buf.push_varint_uint64(2);

            buf.end_len();

            // b: string
            buf.push_field_len(2);
            buf.begin_len(());
            buf.push(b"Some text");
            buf.end_len();

            // c: int64
            buf.push_field_varint(3);
            buf.push_varint_uint64(2);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            pub struct Nested<'a> {
                a: NestedInner<'a>,
                b: &'a str,
                c: i32,
            }

            #[derive(Value)]
            pub struct NestedInner<'a> {
                a: BasicOptional,
                b: &'a sval::BinarySlice,
                c: i32,
            }

            #[derive(Value)]
            pub struct BasicOptional {
                a: Option<i32>,
            }

            sval_protobuf::stream_to_protobuf(Nested {
                a: NestedInner {
                    a: BasicOptional { a: Some(1) },
                    b: sval::BinarySlice::new(b"Some bytes"),
                    c: 2,
                },
                b: "Some text",
                c: 2,
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&prost, &raw);
        assert_proto(&prost, &sval);
    }

    #[test]
    fn exotic_enum_tuple() {
        #[derive(Value)]
        #[allow(dead_code)]
        pub enum Enum {
            Tag,
            Tuple(i32, i32),
        }

        #[derive(Value)]
        pub struct Struct {
            a: i32,
            b: Enum,
        }

        let raw = {
            let mut buf = ProtoBufMut::new(());

            // a
            buf.push_field_varint(1);
            buf.push_varint_uint64(2);

            // b
            buf.push_field_len(2);
            buf.begin_len(());

            // Enum::Tuple
            buf.push_field_len(2);
            buf.begin_len(());

            // Enum::Tuple.0
            buf.push_field_varint(1);
            buf.push_varint_uint64(3);

            // Enum::Tuple.1
            buf.push_field_varint(2);
            buf.push_varint_uint64(4);

            buf.end_len();

            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Struct {
                a: 2,
                b: Enum::Tuple(3, 4),
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn exotic_tagged_empty() {
        struct Tagged;

        impl sval::Value for Tagged {
            fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> sval::Result {
                stream.tagged_begin(None, Some(&sval::Label::new("Tagged")), None)?;
                stream.tagged_end(None, Some(&sval::Label::new("Tagged")), None)
            }
        }

        #[derive(Value)]
        pub struct Struct {
            a: i32,
            b: Tagged,
            c: i32,
        }

        let raw = {
            let mut buf = ProtoBufMut::new(());

            // a
            buf.push_field_varint(1);
            buf.push_varint_uint64(2);

            // c
            buf.push_field_varint(3);
            buf.push_varint_uint64(5);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Struct {
                a: 2,
                b: Tagged,
                c: 5,
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&raw, &sval);

        let raw = {
            let buf = ProtoBufMut::new(());

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Tagged)
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn exotic_enum_empty() {
        struct Enum;

        impl sval::Value for Enum {
            fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> sval::Result {
                stream.enum_begin(None, Some(&sval::Label::new("Enum")), None)?;
                stream.enum_end(None, Some(&sval::Label::new("Enum")), None)
            }
        }

        #[derive(Value)]
        pub struct Struct {
            a: i32,
            b: Enum,
            c: i32,
        }

        let raw = {
            let mut buf = ProtoBufMut::new(());

            // a
            buf.push_field_varint(1);
            buf.push_varint_uint64(2);

            // c
            buf.push_field_varint(3);
            buf.push_varint_uint64(5);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Struct {
                a: 2,
                b: Enum,
                c: 5,
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&raw, &sval);

        let raw = {
            let buf = ProtoBufMut::new(());

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Enum)
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn exotic_nested_enum_empty() {
        struct Outer;

        impl sval::Value for Outer {
            fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> sval::Result {
                stream.enum_begin(None, Some(&sval::Label::new("Outer")), None)?;
                stream.enum_begin(
                    None,
                    Some(&sval::Label::new("Inner")),
                    Some(&sval::Index::new(0).with_tag(&sval::tags::VALUE_OFFSET)),
                )?;
                stream.enum_begin(
                    None,
                    Some(&sval::Label::new("Core")),
                    Some(&sval::Index::new(0).with_tag(&sval::tags::VALUE_OFFSET)),
                )?;
                stream.enum_end(
                    None,
                    Some(&sval::Label::new("Core")),
                    Some(&sval::Index::new(0).with_tag(&sval::tags::VALUE_OFFSET)),
                )?;
                stream.enum_end(
                    None,
                    Some(&sval::Label::new("Inner")),
                    Some(&sval::Index::new(0).with_tag(&sval::tags::VALUE_OFFSET)),
                )?;
                stream.enum_end(None, Some(&sval::Label::new("Outer")), None)
            }
        }

        #[derive(Value)]
        pub struct Struct {
            a: i32,
            b: Outer,
            c: i32,
        }

        let raw = {
            let mut buf = ProtoBufMut::new(());

            // a
            buf.push_field_varint(1);
            buf.push_varint_uint64(2);

            // b
            buf.push_field_len(2);
            buf.begin_len(());

            buf.push_field_len(1);
            buf.begin_len(());
            buf.end_len();

            buf.end_len();

            // c
            buf.push_field_varint(3);
            buf.push_varint_uint64(5);

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Struct {
                a: 2,
                b: Outer,
                c: 5,
            })
            .to_vec()
            .into_owned()
        };

        assert_proto(&raw, &sval);

        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);

            buf.begin_len(());
            buf.end_len();

            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            sval_protobuf::stream_to_protobuf(Outer)
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn pre_encoded_len() {
        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());

            buf.push_field_varint(1);
            buf.push_varint_uint64(1);

            buf.push_field_varint(2);
            buf.push_varint_uint64(2);

            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            struct Outer<'a> {
                inner: &'a ProtoBuf,
            }

            #[derive(Value)]
            struct Inner {
                a: u64,
                b: u64,
            }

            let inner = sval_protobuf::stream_to_protobuf(Inner { a: 1, b: 2 });

            sval_protobuf::stream_to_protobuf(Outer { inner: &inner })
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn pre_encoded_varint() {
        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());

            buf.push_field_varint(1);
            buf.push_varint_uint64(42);

            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            struct Outer<'a> {
                inner: &'a ProtoBuf,
            }

            let inner = sval_protobuf::stream_to_protobuf(42);

            sval_protobuf::stream_to_protobuf(Outer { inner: &inner })
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn pre_encoded_i32() {
        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());

            buf.push_field_i32(1);
            buf.push_i32_float(3.14);

            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            struct Outer<'a> {
                inner: &'a ProtoBuf,
            }

            let inner = sval_protobuf::stream_to_protobuf(3.14f32);

            sval_protobuf::stream_to_protobuf(Outer { inner: &inner })
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }

    #[test]
    fn pre_encoded_i64() {
        let raw = {
            let mut buf = ProtoBufMut::new(());

            buf.push_field_len(1);
            buf.begin_len(());

            buf.push_field_i64(1);
            buf.push_i64_double(3.1415);

            buf.end_len();

            buf.freeze().to_vec().into_owned()
        };

        let sval = {
            #[derive(Value)]
            struct Outer<'a> {
                inner: &'a ProtoBuf,
            }

            let inner = sval_protobuf::stream_to_protobuf(3.1415f64);

            sval_protobuf::stream_to_protobuf(Outer { inner: &inner })
                .to_vec()
                .into_owned()
        };

        assert_proto(&raw, &sval);
    }
}

#[track_caller]
#[cfg(test)]
fn assert_proto(expected: &[u8], actual: &[u8]) {
    let roundtrip = sval_protobuf::buf::ProtoBuf::pre_encoded(actual);
    assert_eq!(actual, &*roundtrip.to_vec());

    assert_eq!(
        expected,
        actual,
        "\nexpected:\n{}\nactual:\n{}",
        inspect(&expected),
        inspect(&actual)
    )
}

#[cfg(test)]
fn inspect(encoded: &[u8]) -> String {
    protoscope(encoded).unwrap_or_else(|_| "<protoscope not available>".to_owned())
}

#[cfg(test)]
fn protoscope(encoded: &[u8]) -> Result<String, Box<dyn std::error::Error + 'static>> {
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
    };

    let mut protoscope = Command::new("protoscope")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = protoscope.stdin.take().ok_or("missing stdin")?;
    stdin.write_all(encoded)?;
    drop(stdin);

    let mut buf = String::new();
    protoscope
        .stdout
        .take()
        .ok_or("missing stdout")?
        .read_to_string(&mut buf)?;

    Ok(buf)
}
