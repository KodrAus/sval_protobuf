/*!
Buffering writer for protobuf.

The [`ProtoBufMut`] type can be used to efficiently encode a protobuf
value without necessarily knowing the final size upfront.
*/

use crate::raw::{VarInt, WireType, I32, I64};
use alloc::{borrow::Cow, boxed::Box, vec::Vec};

pub(crate) const APPROXIMATE_DEPTH: usize = 16;

mod cursor;
mod visit;

pub use self::cursor::*;

/**
Buffering writer for protobuf, with state `T`.

The writer uses a stack to track the lengths of nested length-prefixed fields.
Each frame in the stack will use its own instance of `T`.
*/
#[derive(Debug)]
pub struct ProtoBufMut<T> {
    bytes: Vec<u8>,
    chunks: Vec<LenPrefixedChunk>,
    root_state: T,
    len_stack: Vec<LenStackFrame<T>>,
}

/**
An encoded protobuf value.

`ProtoBuf`s can be used directly as nested messages in larger messages.
*/
#[derive(Clone, Debug)]
pub struct ProtoBuf {
    bytes: Box<[u8]>,
    chunks: Box<[LenPrefixedChunk]>,
}

#[derive(Debug)]
struct LenStackFrame<T> {
    len: usize,
    head: usize,
    chunk_idx: usize,
    state: T,
}

#[derive(Debug, Clone, Copy)]
struct LenPrefixedChunk {
    // Written before the data in `range`
    varint: Option<u64>,
    // The index to write from
    // The end is the start of the following chunk
    start: usize,
}

impl<T> ProtoBufMut<T> {
    /**
    Create a new buffering writer, with initial state `T`.
    */
    #[inline(always)]
    pub fn new(state: T) -> Self {
        ProtoBufMut {
            bytes: Vec::new(),
            chunks: Vec::new(),
            root_state: state,
            len_stack: Vec::with_capacity(APPROXIMATE_DEPTH),
        }
    }

    /**
    The current depth of the length-prefixed stack.
    */
    #[inline(always)]
    pub fn depth(&self) -> usize {
        self.len_stack.len()
    }

    /**
    Encode a value using variable-length encoding.
    */
    #[inline(always)]
    pub fn push_varint(&mut self, v: VarInt) {
        self.push(v.fill_bytes(&mut [0; 10]));
    }

    /**
    Encode a 64bit unsigned value using variable-length encoding.
    */
    #[inline(always)]
    pub fn push_varint_uint64(&mut self, v: u64) {
        self.push_varint(VarInt::uint64(v));
    }

    /**
    Encode a 64bit signed value using variable-length encoding.
    */
    #[inline(always)]
    pub fn push_varint_sint64(&mut self, v: i64) {
        self.push_varint(VarInt::sint64(v));
    }

    /**
    Encode a 64bit signed value using variable-length zigzag encoding.
    */
    #[inline(always)]
    pub fn push_varint_sint64z(&mut self, v: i64) {
        self.push_varint(VarInt::sint64z(v));
    }

    /**
    Encode a boolean.
    */
    #[inline(always)]
    pub fn push_varint_bool(&mut self, v: bool) {
        self.push_varint(VarInt::bool(v));
    }

    /**
    Encode a 32bit enum variant tag.
    */
    #[inline(always)]
    pub fn push_varint_enum32(&mut self, v: i32) {
        self.push_varint(VarInt::enum32(v));
    }

    /**
    Encode 32bits.
    */
    #[inline(always)]
    pub fn push_i32(&mut self, v: I32) {
        self.push(&v.to_bytes());
    }

    /**
    Encode a 32bit binary floating point value using fixed-length encoding.
    */
    #[inline(always)]
    pub fn push_i32_float(&mut self, v: f32) {
        self.push_i32(I32::float(v));
    }

    /**
    Encode a 32bit unsigned value using fixed-length encoding.
    */
    #[inline(always)]
    pub fn push_i32_fixed32(&mut self, v: u32) {
        self.push_i32(I32::fixed32(v));
    }

    /**
    Encode a 32bit signed value using fixed-length encoding.
    */
    #[inline(always)]
    pub fn push_i32_sfixed32(&mut self, v: i32) {
        self.push_i32(I32::sfixed32(v));
    }

    /**
    Encode 64bits.
    */
    #[inline(always)]
    pub fn push_i64(&mut self, v: I64) {
        self.push(&v.to_bytes());
    }

    /**
    Encode a 64bit binary floating point value using fixed-length encoding.
    */
    #[inline(always)]
    pub fn push_i64_double(&mut self, v: f64) {
        self.push_i64(I64::double(v));
    }

    /**
    Encode a 64bit unsigned value using fixed-length encoding.
    */
    #[inline(always)]
    pub fn push_i64_fixed64(&mut self, v: u64) {
        self.push_i64(I64::fixed64(v));
    }

    /**
    Encode a 64bit signed value using fixed-length encoding.
    */
    #[inline(always)]
    pub fn push_i64_sfixed64(&mut self, v: i64) {
        self.push_i64(I64::sfixed64(v));
    }

    /**
    Write a binary payload.
    */
    #[inline(always)]
    pub fn push(&mut self, b: &[u8]) {
        self.bytes.extend_from_slice(b);
    }

    /**
    Encode the header for a field.
    */
    #[inline(always)]
    pub fn push_field(&mut self, field_number: u64, wire_type: WireType) {
        self.push_varint(VarInt::field(field_number, wire_type));
    }

    /**
    Encode the header for a variable-length encoded field.
    */
    #[inline(always)]
    pub fn push_field_varint(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::VarInt);
    }

    /**
    Encode the header for a 64bit fixed-length encoded field.
    */
    #[inline(always)]
    pub fn push_field_i64(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::I64);
    }

    /**
    Encode the header for a 32bit fixed-length encoded field.
    */
    #[inline(always)]
    pub fn push_field_i32(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::I32);
    }

    /**
    Encode the header for a length-prefixed field.

    This method should be immediately followed by a call to [`ProtoBufMut::push_len_varint_uint64`]
    or [`ProtoBufMut::begin_len`].
    */
    #[inline(always)]
    pub fn push_field_len(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::Len);
    }

    /**
    Encode the length of a length-prefixed field.
    */
    #[inline(always)]
    pub fn push_len_varint_uint64(&mut self, len: u64) {
        self.push_varint_uint64(len);
    }

    #[inline]
    pub(crate) fn reserve(&mut self, num_entries: usize) {
        self.bytes.reserve((256 * num_entries) / (self.depth() + 1));
    }

    #[inline]
    pub(crate) fn reserve_bytes(&mut self, num_bytes: usize) {
        self.bytes.reserve(num_bytes);
    }

    /**
    Begin a new length-prefixed value, where the length isn't known upfront.

    This method accepts a new instance of state `T` that will be associated with this value.
    Once the value has been encoded, call [`ProtoBufMut::end_len`] to complete it.
    */
    pub fn begin_len(&mut self, state: T) {
        // If there is an active message already then perform some bookkeeping
        // Track any bytes written in the parent up to this point in its length
        // The head will stay at the start of this field until we finish it
        if let Some(parent) = self.len_stack.last_mut() {
            parent.len += self.bytes.len() - parent.head;
            parent.head = self.bytes.len();
        }

        // Push some state to the stack for this length-prefixed field
        // It will track the length and the corresponding chunk to prefix
        // that length with once it's known
        self.len_stack.push(LenStackFrame {
            len: 0,
            head: self.bytes.len(),
            chunk_idx: self.chunks.len(),
            state,
        });

        // Add the chunk that will carry the length of this field
        self.chunks.push(LenPrefixedChunk {
            varint: None,
            start: self.bytes.len(),
        });
    }

    /**
    Get the state at the current depth.
    */
    pub fn state_mut(&mut self) -> &mut T {
        self.len_stack
            .last_mut()
            .map(|frame| &mut frame.state)
            .unwrap_or(&mut self.root_state)
    }

    /**
    Complete a length-prefixed value, where the length wasn't known upfront.
    */
    pub fn end_len(&mut self) {
        if let Some(frame) = self.len_stack.pop() {
            // Calculate any remaining unaccounted for bytes
            let len = frame.len + (self.bytes.len() - frame.head);

            // Set the varint value in the chunk
            self.chunks[frame.chunk_idx].varint = Some(len as u64);

            // If there is an active message already then perform some bookkeeping
            // This is the same as when starting a length-prefixed field
            // We don't need to use the parent's head value though, because we've
            // already accounted for all those bytes in the field's `len`
            if let Some(parent) = self.len_stack.last_mut() {
                parent.len += len + VarInt::uint64(len as u64).len();
                parent.head = self.bytes.len();
            }
        }
    }

    /**
    Complete the writer, returning an immutable buffer containing the encoded protobuf payload.
    */
    #[inline(always)]
    pub fn freeze(self) -> ProtoBuf {
        ProtoBuf {
            bytes: self.bytes.into_boxed_slice(),
            chunks: self.chunks.into_boxed_slice(),
        }
    }
}

impl ProtoBuf {
    /**
    Treat a buffer as a pre-encoded message.

    No validation is performed on the given buffer; it's expected to already
    contain a valid message.
    */
    pub fn pre_encoded(buf: impl Into<Box<[u8]>>) -> Self {
        ProtoBuf { bytes: buf.into(), chunks: [].into() }
    }

    /**
    Get the length in bytes of the encoded payload.
    */
    pub fn len(&self) -> usize {
        visit::len(&self.bytes, &self.chunks)
    }

    /**
    Get the payload as a contiguous buffer.
    */
    pub fn to_vec(&self) -> Cow<[u8]> {
        visit::to_vec(&self.bytes, &self.chunks)
    }

    /**
    Convert the payload into a reader that will yield its contents without potentially copying them first.
    */
    pub fn into_cursor(self) -> ProtoBufCursor {
        ProtoBufCursor::new(self.bytes, self.chunks)
    }
}

impl sval::Value for ProtoBuf {
    fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> sval::Result {
        visit::to_stream(&self.bytes, &self.chunks, stream)
    }
}
