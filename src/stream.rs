use crate::buf::{ProtoBuf, ProtoBufMut};
use crate::raw::WireType;
use crate::tags;
use sval::{Index, Label, Tag};

/**
Encode a value to the protobuf wire format.

Standalone scalar values will be wrapped in a message with a field number `1`.
*/
pub fn stream_to_protobuf(v: impl sval::Value) -> ProtoBuf {
    let mut stream = ProtoBufStream {
        buf: ProtoBufMut::new(1),
        field: FieldState {
            number: 1,
            default_null: true,
            ty: FieldType::Root,
        },
        len: LenState {
            is_packed: false,
            is_prefixed: false,
        },
        one_of: OneOfState {
            is_internally_tagged: false,
        },
    };

    let _ = v.stream(&mut stream);

    stream.buf.freeze()
}

#[derive(Debug)]
struct ProtoBufStream {
    buf: ProtoBufMut<u64>,
    field: FieldState,
    len: LenState,
    one_of: OneOfState,
}

#[derive(Debug)]
struct FieldState {
    number: u64,
    default_null: bool,
    ty: FieldType,
}

#[derive(Debug)]
enum FieldType {
    Any,
    Root,
    Signed,
    I32,
    I64,
}

impl FieldState {
    #[inline(always)]
    fn is_set(&self) -> bool {
        self.number != 0
    }

    #[inline]
    fn set(&mut self, index: &Index) {
        self.ty = FieldType::Any;
        self.number = match index.tag() {
            // Field indexes are 1-based in protobuf, but 0-based in sval
            // If the index came from a Rust field offset then increment it
            Some(&sval::tags::VALUE_OFFSET) => index.to_u64().unwrap_or_default() + 1,
            // If the index was specified then use it directly
            _ => index.to_u64().unwrap_or(1),
        }
    }

    #[inline(always)]
    fn push_if_set<T>(&mut self, wire_type: WireType, buf: &mut ProtoBufMut<T>) {
        if self.is_set() {
            self.push(wire_type, buf)
        }
    }

    #[inline(always)]
    fn push<T>(&mut self, wire_type: WireType, buf: &mut ProtoBufMut<T>) {
        buf.push_field(self.number, wire_type);
        self.number = 0;
    }
}

#[derive(Debug)]
struct LenState {
    is_packed: bool,
    is_prefixed: bool,
}

#[derive(Debug)]
struct OneOfState {
    is_internally_tagged: bool,
}

impl ProtoBufStream {
    fn internally_tagged_begin(&mut self, index: Option<&Index>) {
        if self.one_of.is_internally_tagged {
            self.one_of.is_internally_tagged = false;

            if self.field.is_set() {
                self.field.push(WireType::Len, &mut self.buf);
                self.buf.begin_len(1);
            }

            if let Some(index) = index {
                self.field.set(index);
            }
        }
    }

    fn internally_tagged_end(&mut self, index: Option<&Index>) {
        if index.is_some() {
            self.one_of.is_internally_tagged = true;
        }
    }

    fn root_begin(&mut self) {
        if let FieldType::Root = self.field.ty {
            self.field = FieldState {
                ty: FieldType::Any,
                number: 0,
                default_null: true,
            }
        }
    }

    fn field_begin(&mut self) {
        self.one_of.is_internally_tagged = false;
    }
}

impl<'sval> sval::Stream<'sval> for ProtoBufStream {
    #[inline]
    fn null(&mut self) -> sval::Result {
        self.field.number = 0;

        Ok(())
    }

    fn bool(&mut self, value: bool) -> sval::Result {
        if self.field.default_null && value == false {
            self.null()
        } else {
            self.field.push_if_set(WireType::VarInt, &mut self.buf);
            self.buf.push_varint_bool(value);

            Ok(())
        }
    }

    fn text_begin(&mut self, num_bytes: Option<usize>) -> sval::Result {
        self.binary_begin(num_bytes)
    }

    fn text_fragment_computed(&mut self, fragment: &str) -> sval::Result {
        self.binary_fragment_computed(fragment.as_bytes())
    }

    fn text_end(&mut self) -> sval::Result {
        self.binary_end()
    }

    fn binary_begin(&mut self, num_bytes: Option<usize>) -> sval::Result {
        if let Some(num_bytes) = num_bytes {
            self.len.is_prefixed = true;

            if self.field.default_null && num_bytes == 0 {
                self.null()
            } else {
                self.buf.reserve_bytes(num_bytes);

                self.field.push_if_set(WireType::Len, &mut self.buf);
                self.buf.push_len_varint_uint64(num_bytes as u64);

                Ok(())
            }
        } else {
            self.field.push_if_set(WireType::Len, &mut self.buf);
            self.buf.begin_len(1);

            Ok(())
        }
    }

    fn binary_fragment_computed(&mut self, fragment: &[u8]) -> sval::Result {
        self.buf.push(fragment);

        Ok(())
    }

    fn binary_end(&mut self) -> sval::Result {
        if self.len.is_prefixed {
            self.len.is_prefixed = false;

            Ok(())
        } else {
            self.buf.end_len();

            Ok(())
        }
    }

    fn u32(&mut self, value: u32) -> sval::Result {
        if self.field.default_null && value == 0 {
            self.null()
        } else {
            match self.field.ty {
                FieldType::I32 => {
                    self.field.push_if_set(WireType::I32, &mut self.buf);
                    self.buf.push_i32_fixed32(value);

                    Ok(())
                }
                _ => {
                    self.field.push_if_set(WireType::VarInt, &mut self.buf);
                    self.buf.push_varint_uint64(value as u64);

                    Ok(())
                }
            }
        }
    }

    fn u64(&mut self, value: u64) -> sval::Result {
        if self.field.default_null && value == 0 {
            self.null()
        } else {
            match self.field.ty {
                FieldType::I64 => {
                    self.field.push_if_set(WireType::I64, &mut self.buf);
                    self.buf.push_i64_fixed64(value);

                    Ok(())
                }
                _ => {
                    self.field.push_if_set(WireType::VarInt, &mut self.buf);
                    self.buf.push_varint_uint64(value);

                    Ok(())
                }
            }
        }
    }

    fn u128(&mut self, value: u128) -> sval::Result {
        let bytes = value.to_le_bytes();

        self.binary_begin(Some(bytes.len()))?;
        self.binary_fragment_computed(&bytes)?;
        self.binary_end()
    }

    fn i32(&mut self, value: i32) -> sval::Result {
        if self.field.default_null && value == 0 {
            self.null()
        } else {
            match self.field.ty {
                FieldType::I32 => {
                    self.field.push_if_set(WireType::I32, &mut self.buf);
                    self.buf.push_i32_sfixed32(value);

                    Ok(())
                }
                FieldType::Signed => {
                    self.field.push_if_set(WireType::VarInt, &mut self.buf);
                    self.buf.push_varint_sint64z(value as i64);

                    Ok(())
                }
                _ => {
                    self.field.push_if_set(WireType::VarInt, &mut self.buf);
                    self.buf.push_varint_sint64(value as i64);

                    Ok(())
                }
            }
        }
    }

    fn i64(&mut self, value: i64) -> sval::Result {
        if self.field.default_null && value == 0 {
            self.null()
        } else {
            match self.field.ty {
                FieldType::I64 => {
                    self.field.push_if_set(WireType::I64, &mut self.buf);
                    self.buf.push_i64_sfixed64(value);

                    Ok(())
                }
                FieldType::Signed => {
                    self.field.push_if_set(WireType::VarInt, &mut self.buf);
                    self.buf.push_varint_sint64z(value);

                    Ok(())
                }
                _ => {
                    self.field.push_if_set(WireType::VarInt, &mut self.buf);
                    self.buf.push_varint_sint64(value);

                    Ok(())
                }
            }
        }
    }

    fn i128(&mut self, value: i128) -> sval::Result {
        let bytes = value.to_le_bytes();

        self.binary_begin(Some(bytes.len()))?;
        self.binary_fragment_computed(&bytes)?;
        self.binary_end()
    }

    fn f32(&mut self, value: f32) -> sval::Result {
        if self.field.default_null && value == 0.0 {
            self.null()
        } else {
            self.field.push_if_set(WireType::I32, &mut self.buf);
            self.buf.push_i32_float(value);

            Ok(())
        }
    }

    fn f64(&mut self, value: f64) -> sval::Result {
        if self.field.default_null && value == 0.0 {
            self.null()
        } else {
            self.field.push_if_set(WireType::I64, &mut self.buf);
            self.buf.push_i64_double(value);

            Ok(())
        }
    }

    fn map_begin(&mut self, num_entries: Option<usize>) -> sval::Result {
        if let Some(num_entries) = num_entries {
            if self.field.default_null && num_entries == 0 {
                self.len.is_prefixed = true;

                return self.null();
            }

            self.buf.reserve(num_entries * 2);
        }

        *self.buf.state_mut() = self.field.number;

        self.field.number = 0;
        self.field.ty = FieldType::Any;

        Ok(())
    }

    fn map_key_begin(&mut self) -> sval::Result {
        self.field_begin();

        self.field.number = *self.buf.state_mut();
        self.field.push(WireType::Len, &mut self.buf);
        self.field.number = 1;

        self.buf.begin_len(1);

        Ok(())
    }

    fn map_key_end(&mut self) -> sval::Result {
        Ok(())
    }

    fn map_value_begin(&mut self) -> sval::Result {
        self.field.number = 2;

        Ok(())
    }

    fn map_value_end(&mut self) -> sval::Result {
        self.buf.end_len();

        Ok(())
    }

    fn map_end(&mut self) -> sval::Result {
        self.len.is_prefixed = false;

        Ok(())
    }

    fn seq_begin(&mut self, num_entries: Option<usize>) -> sval::Result {
        if let Some(num_entries) = num_entries {
            if self.field.default_null && num_entries == 0 {
                self.len.is_prefixed = true;

                return self.null();
            }

            self.buf.reserve(num_entries);
        }

        if self.len.is_packed {
            self.field.push_if_set(WireType::Len, &mut self.buf);
            self.buf.begin_len(1);

            Ok(())
        } else {
            *self.buf.state_mut() = self.field.number;

            self.field.number = 0;
            self.field.ty = FieldType::Any;

            Ok(())
        }
    }

    fn seq_value_begin(&mut self) -> sval::Result {
        self.field.default_null = false;

        if self.len.is_packed {
            Ok(())
        } else {
            self.field_begin();
            self.field.number = *self.buf.state_mut();

            Ok(())
        }
    }

    fn seq_value_end(&mut self) -> sval::Result {
        self.field.default_null = true;

        Ok(())
    }

    fn seq_end(&mut self) -> sval::Result {
        self.len.is_prefixed = false;

        if self.len.is_packed {
            self.len.is_packed = false;

            self.buf.end_len();

            Ok(())
        } else {
            Ok(())
        }
    }

    fn enum_begin(
        &mut self,
        _: Option<&Tag>,
        _: Option<&Label>,
        index: Option<&Index>,
    ) -> sval::Result {
        self.root_begin();
        self.internally_tagged_begin(index);

        self.one_of.is_internally_tagged = true;

        Ok(())
    }

    fn enum_end(
        &mut self,
        _: Option<&Tag>,
        _: Option<&Label>,
        index: Option<&Index>,
    ) -> sval::Result {
        if self.one_of.is_internally_tagged {
            self.one_of.is_internally_tagged = false;

            self.buf.end_len();
        }

        self.internally_tagged_end(index);

        Ok(())
    }

    fn tagged_begin(
        &mut self,
        tag: Option<&Tag>,
        _: Option<&Label>,
        index: Option<&Index>,
    ) -> sval::Result {
        self.internally_tagged_begin(index);

        match tag {
            Some(&tags::PROTOBUF_I32) => {
                self.field.ty = FieldType::I32;

                Ok(())
            }
            Some(&tags::PROTOBUF_I64) => {
                self.field.ty = FieldType::I64;

                Ok(())
            }
            Some(&tags::PROTOBUF_LEN_PACKED) => {
                self.len.is_packed = true;

                Ok(())
            }
            Some(&tags::PROTOBUF_VARINT_SIGNED) => {
                self.field.ty = FieldType::Signed;

                Ok(())
            }
            Some(&sval::tags::RUST_OPTION_SOME) => {
                self.field.default_null = false;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn tagged_end(
        &mut self,
        _: Option<&Tag>,
        _: Option<&Label>,
        index: Option<&Index>,
    ) -> sval::Result {
        self.internally_tagged_end(index);

        self.field.ty = FieldType::Any;
        self.field.default_null = true;

        Ok(())
    }

    fn tag(&mut self, tag: Option<&Tag>, _: Option<&Label>, index: Option<&Index>) -> sval::Result {
        self.one_of.is_internally_tagged = false;

        match tag {
            Some(&sval::tags::RUST_OPTION_NONE) => self.null(),
            _ => {
                // Protobuf enums are i32 values
                if let Some(index) = index.and_then(|index| index.to_i32()) {
                    self.i32(index)
                } else {
                    self.null()
                }
            }
        }
    }

    fn tuple_begin(
        &mut self,
        _: Option<&Tag>,
        _: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        if let Some(num_entries) = num_entries {
            self.buf.reserve(num_entries);
        }

        self.root_begin();
        self.internally_tagged_begin(index);

        if self.field.is_set() {
            self.field.push(WireType::Len, &mut self.buf);
            self.buf.begin_len(1);
        }

        Ok(())
    }

    fn tuple_value_begin(&mut self, _: Option<&Tag>, index: &Index) -> sval::Result {
        self.field_begin();

        self.field.set(index);

        Ok(())
    }

    fn tuple_value_end(&mut self, _: Option<&Tag>, _: &Index) -> sval::Result {
        Ok(())
    }

    fn tuple_end(
        &mut self,
        _: Option<&Tag>,
        _: Option<&Label>,
        index: Option<&Index>,
    ) -> sval::Result {
        self.internally_tagged_end(index);

        // The root message isn't wrapped
        if self.buf.depth() != 0 {
            self.buf.end_len();
        }

        Ok(())
    }

    fn record_tuple_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        self.tuple_begin(tag, label, index, num_entries)
    }

    fn record_tuple_value_begin(
        &mut self,
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> sval::Result {
        let _ = label;

        self.tuple_value_begin(tag, index)
    }

    fn record_tuple_value_end(
        &mut self,
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> sval::Result {
        let _ = label;

        self.tuple_value_end(tag, index)
    }

    fn record_tuple_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> sval::Result {
        self.tuple_end(tag, label, index)
    }
}
