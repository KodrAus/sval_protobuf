/*!
Buffering writer for protobuf.

The [`ProtoBufMut`] type can be used to efficiently encode a protobuf
value without necessarily knowing the final size upfront.
*/

use crate::raw::{VarInt, WireType, I32, I64};
use alloc::{borrow::Cow, boxed::Box, vec::Vec};
use core::{cmp, ops::Deref, slice};

pub(crate) const APPROXIMATE_DEPTH: usize = 16;

/**
Buffering writer for protobuf, with state `T`.
*/
#[derive(Debug)]
pub struct ProtoBufMut<T> {
    bytes: Vec<u8>,
    chunks: Vec<LenPrefixedChunk>,
    approximate_len_bytes: usize,
    root_state: T,
    len_stack: Vec<LenStackFrame<T>>,
}

/**
An encoded protobuf value.

`ProtoBuf`s can be used as nested messages in larger messages.
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
    #[inline(always)]
    pub fn new(state: T) -> Self {
        ProtoBufMut {
            approximate_len_bytes: 128,
            bytes: Vec::new(),
            chunks: Vec::new(),
            root_state: state,
            len_stack: Vec::with_capacity(APPROXIMATE_DEPTH),
        }
    }

    #[inline(always)]
    pub fn depth(&self) -> usize {
        self.len_stack.len()
    }

    #[inline(always)]
    pub fn push_varint(&mut self, v: VarInt) {
        self.push(v.fill_bytes(&mut [0; 10]));
    }

    #[inline(always)]
    pub fn push_varint_uint64(&mut self, v: u64) {
        self.push_varint(VarInt::uint64(v));
    }

    #[inline(always)]
    pub fn push_varint_sint64(&mut self, v: i64) {
        self.push_varint(VarInt::sint64(v));
    }

    #[inline(always)]
    pub fn push_varint_sint64z(&mut self, v: i64) {
        self.push_varint(VarInt::sint64z(v));
    }

    #[inline(always)]
    pub fn push_varint_bool(&mut self, v: bool) {
        self.push_varint(VarInt::bool(v));
    }

    #[inline(always)]
    pub fn push_varint_enum32(&mut self, v: i32) {
        self.push_varint(VarInt::enum32(v));
    }

    #[inline(always)]
    pub fn push_i32(&mut self, v: I32) {
        self.push(&v.to_bytes());
    }

    #[inline(always)]
    pub fn push_i32_float(&mut self, v: f32) {
        self.push_i32(I32::float(v));
    }

    #[inline(always)]
    pub fn push_i32_fixed32(&mut self, v: u32) {
        self.push_i32(I32::fixed32(v));
    }

    #[inline(always)]
    pub fn push_i32_sfixed32(&mut self, v: i32) {
        self.push_i32(I32::sfixed32(v));
    }

    #[inline(always)]
    pub fn push_i64(&mut self, v: I64) {
        self.push(&v.to_bytes());
    }

    #[inline(always)]
    pub fn push_i64_double(&mut self, v: f64) {
        self.push_i64(I64::double(v));
    }

    #[inline(always)]
    pub fn push_i64_fixed64(&mut self, v: u64) {
        self.push_i64(I64::fixed64(v));
    }

    #[inline(always)]
    pub fn push_i64_sfixed64(&mut self, v: i64) {
        self.push_i64(I64::sfixed64(v));
    }

    #[inline(always)]
    pub fn push(&mut self, b: &[u8]) {
        self.bytes.extend_from_slice(b);
    }

    #[inline(always)]
    pub fn push_field(&mut self, field_number: u64, wire_type: WireType) {
        self.push_varint(VarInt::field(field_number, wire_type));
    }

    #[inline(always)]
    pub fn push_field_varint(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::VarInt);
    }

    #[inline(always)]
    pub fn push_field_i64(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::I64);
    }

    #[inline(always)]
    pub fn push_field_i32(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::I32);
    }

    #[inline(always)]
    pub fn push_field_len(&mut self, field_number: u64) {
        self.push_field(field_number, WireType::Len);
    }

    #[inline(always)]
    pub fn push_len_varint_uint64(&mut self, len: u64) {
        self.push_varint_uint64(len);
    }

    #[inline(always)]
    fn observe_len_bytes(&mut self, len: usize) {
        let len = cmp::min(len, 1024 * 8);

        self.approximate_len_bytes = cmp::max(self.approximate_len_bytes, len);
    }

    pub(crate) fn reserve(&mut self, num_entries: usize) {
        // NOTE: This may result in constant over-allocation; we might want
        // to tweak this based to only pre-allocate on "big" looking messages
        self.bytes.reserve(self.approximate_len_bytes);
        self.chunks.reserve(num_entries);
    }

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

    pub fn state_mut(&mut self) -> &mut T {
        self.len_stack
            .last_mut()
            .map(|frame| &mut frame.state)
            .unwrap_or(&mut self.root_state)
    }

    pub fn end_len(&mut self) {
        if let Some(frame) = self.len_stack.pop() {
            // Calculate any remaining unaccounted for bytes
            let len = frame.len + (self.bytes.len() - frame.head);

            self.observe_len_bytes(len);

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

    pub fn iter_chunks(&self) -> IterChunks {
        IterChunks::new(&self.bytes, &self.chunks)
    }

    pub fn to_vec(&self) -> Cow<[u8]> {
        to_vec(&self.bytes, &self.chunks)
    }

    #[inline(always)]
    pub fn freeze(self) -> ProtoBuf {
        ProtoBuf {
            bytes: self.bytes.into_boxed_slice(),
            chunks: self.chunks.into_boxed_slice(),
        }
    }
}

impl ProtoBuf {
    pub fn iter_chunks(&self) -> IterChunks {
        IterChunks::new(&self.bytes, &self.chunks)
    }

    pub fn to_vec(&self) -> Cow<[u8]> {
        to_vec(&self.bytes, &self.chunks)
    }
}

impl sval::Value for ProtoBuf {
    fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> sval::Result {
        to_stream(&self.bytes, &self.chunks, stream)
    }
}

fn to_stream<'a>(
    bytes: &'a [u8],
    chunks: &'a [LenPrefixedChunk],
    stream: &mut (impl sval::Stream<'a> + ?Sized),
) -> sval::Result {
    if chunks.len() == 0 {
        stream.binary_begin(Some(bytes.len()))?;
        stream.binary_fragment(bytes)?;
    } else {
        stream.binary_begin(None)?;

        for chunk in IterChunks::new(bytes, chunks) {
            if let Some(bytes) = chunk.as_borrowed_slice() {
                stream.binary_fragment(bytes)?;
            } else {
                stream.binary_fragment_computed(&*chunk)?;
            }
        }
    }

    stream.binary_end()
}

#[inline(always)]
fn to_vec<'a>(bytes: &'a [u8], chunks: &'a [LenPrefixedChunk]) -> Cow<'a, [u8]> {
    if chunks.len() == 0 {
        return Cow::Borrowed(&bytes);
    }

    let mut buf = Vec::new();
    for chunk in IterChunks::new(bytes, chunks) {
        buf.extend_from_slice(&*chunk);
    }

    Cow::Owned(buf)
}

pub struct IterChunks<'a> {
    bytes: &'a [u8],
    iter: slice::Iter<'a, LenPrefixedChunk>,
    state: IterChunksState<'a>,
}

impl<'a> IterChunks<'a> {
    fn new(bytes: &'a [u8], chunks: &'a [LenPrefixedChunk]) -> Self {
        let mut iter = chunks.iter();

        if let Some(chunk) = iter.next() {
            IterChunks {
                bytes,
                iter,
                state: IterChunksState::Chunk(0, chunk),
            }
        } else {
            IterChunks {
                bytes,
                iter,
                state: IterChunksState::Trailing(0),
            }
        }
    }
}

enum IterChunksState<'a> {
    Chunk(usize, &'a LenPrefixedChunk),
    VarInt(usize, u64),
    Trailing(usize),
    Done,
}

impl<'a> Iterator for IterChunks<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IterChunksState::Chunk(from, chunk) => {
                let to = chunk.start;

                let item = Chunk::bytes(&self.bytes[from..to]);

                if let Some(varint) = chunk.varint {
                    self.state = IterChunksState::VarInt(to, varint);
                } else if let Some(next) = self.iter.next() {
                    self.state = IterChunksState::Chunk(to, next);
                } else {
                    self.state = IterChunksState::Trailing(to);
                }

                Some(item)
            }
            IterChunksState::VarInt(to, varint) => {
                let item = Chunk::varint(VarInt::uint64(varint));

                if let Some(next) = self.iter.next() {
                    self.state = IterChunksState::Chunk(to, next);
                } else {
                    self.state = IterChunksState::Trailing(to);
                }

                Some(item)
            }
            IterChunksState::Trailing(from) => {
                let item = Chunk::bytes(&self.bytes[from..]);

                self.state = IterChunksState::Done;

                Some(item)
            }
            IterChunksState::Done => None,
        }
    }
}

pub struct Chunk<'a>(ChunkInner<'a>);

enum ChunkInner<'a> {
    Bytes(&'a [u8]),
    VarInt([u8; 10], u8),
}

impl<'a> Chunk<'a> {
    fn bytes(slice: &'a [u8]) -> Self {
        Chunk(ChunkInner::Bytes(slice))
    }

    fn varint(varint: VarInt) -> Self {
        let mut buf = [0; 10];
        let len = varint.fill_bytes(&mut buf).len() as u8;

        Chunk(ChunkInner::VarInt(buf, len))
    }

    pub fn as_borrowed_slice(&self) -> Option<&'a [u8]> {
        if let ChunkInner::Bytes(slice) = self.0 {
            Some(slice)
        } else {
            None
        }
    }
}

impl<'a> Deref for Chunk<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self.0 {
            ChunkInner::Bytes(slice) => slice,
            ChunkInner::VarInt(ref buf, len) => &buf[..len as usize],
        }
    }
}

impl<'a> AsRef<[u8]> for Chunk<'a> {
    fn as_ref(&self) -> &[u8] {
        &**self
    }
}
