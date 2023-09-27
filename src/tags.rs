/*!
Tags for protobuf-specific types.
*/

/**
A tag for numeric values that should be encoded in exactly 32bits.
*/
pub const PROTOBUF_I32: sval::Tag = sval::Tag::new("PROTOBUF_I32");

/**
A tag for numeric values that should be encoded in exactly 64bits.
*/
pub const PROTOBUF_I64: sval::Tag = sval::Tag::new("PROTOBUF_I64");

/**
A tag for sequences that should be packed.

This tag is only valid for sequences of numeric values.
*/
pub const PROTOBUF_LEN_PACKED: sval::Tag = sval::Tag::new("PROTOBUF_LEN_PACKED");

/**
A tag for numeric values that should be zigzag encoded.

Variable-length zigzag encoding can be more efficient than the default strategy
for negative values.
*/
pub const PROTOBUF_VARINT_SIGNED: sval::Tag = sval::Tag::new("PROTOBUF_VARINT_SIGNED");
