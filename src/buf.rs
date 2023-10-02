/*!
Buffering writer for protobuf.

The [`ProtoBufMut`] type can be used to efficiently encode a protobuf
value without necessarily knowing the final size upfront.
*/

use crate::raw::{VarInt, WireType, I32, I64};
use alloc::{borrow::Cow, boxed::Box, vec::Vec};
use core::{cmp, ops::Range};

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

    pub(crate) fn reserve_bytes(&mut self, num_bytes: usize) {
        self.bytes.reserve(num_bytes);
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

    pub fn len(&self) -> usize {
        len(&self.bytes, &self.chunks)
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
    pub fn len(&self) -> usize {
        len(&self.bytes, &self.chunks)
    }

    pub fn to_vec(&self) -> Cow<[u8]> {
        to_vec(&self.bytes, &self.chunks)
    }

    pub fn into_cursor(self) -> ProtoBufCursor {
        ProtoBufCursor::new(self.bytes, self.chunks)
    }
}

impl sval::Value for ProtoBuf {
    fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> sval::Result {
        to_stream(&self.bytes, &self.chunks, stream)
    }
}

fn len(bytes: &[u8], chunks: &[LenPrefixedChunk]) -> usize {
    bytes.len()
        + chunks
            .iter()
            .filter_map(|chunk| chunk.varint)
            .map(|varint| VarInt::uint64(varint).len())
            .sum::<usize>()
}

fn to_stream<'a>(
    bytes: &'a [u8],
    chunks: &[LenPrefixedChunk],
    stream: &mut (impl sval::Stream<'a> + ?Sized),
) -> sval::Result {
    if chunks.len() == 0 {
        stream.binary_begin(Some(bytes.len()))?;
        stream.binary_fragment(bytes)?;
    } else {
        stream.binary_begin(Some(len(bytes, chunks)))?;

        struct StreamVisitor<S> {
            stream: S,
            result: sval::Result,
        }

        impl<'sval, S: sval::Stream<'sval>> ChunkVisitor<'sval> for StreamVisitor<S> {
            fn borrowed(&mut self, chunk: &'sval [u8]) {
                self.result = self.stream.binary_fragment(chunk);
            }

            fn computed(&mut self, chunk: &[u8]) {
                self.result = self.stream.binary_fragment_computed(chunk);
            }
        }

        let mut visitor = StreamVisitor {
            stream: &mut *stream,
            result: Ok(()),
        };

        visit_chunks(bytes, chunks, &mut visitor);
        visitor.result?;
    }

    stream.binary_end()
}

#[inline(always)]
fn to_vec<'a>(bytes: &'a [u8], chunks: &[LenPrefixedChunk]) -> Cow<'a, [u8]> {
    if chunks.len() == 0 {
        return Cow::Borrowed(&bytes);
    }

    struct BufVisitor(Vec<u8>);

    impl<'a> ChunkVisitor<'a> for BufVisitor {
        fn computed(&mut self, chunk: &[u8]) {
            self.0.extend_from_slice(chunk);
        }
    }

    let mut visitor = BufVisitor(Vec::with_capacity(len(bytes, chunks)));
    visit_chunks(bytes, chunks, &mut visitor);

    debug_assert_eq!(len(bytes, chunks), visitor.0.len());

    Cow::Owned(visitor.0)
}

trait ChunkVisitor<'a> {
    fn borrowed(&mut self, chunk: &'a [u8]) {
        self.computed(chunk);
    }

    fn computed(&mut self, chunk: &[u8]);
}

impl<'a, 'b, V: ChunkVisitor<'a> + ?Sized> ChunkVisitor<'a> for &'b mut V {
    fn borrowed(&mut self, chunk: &'a [u8]) {
        (**self).borrowed(chunk)
    }

    fn computed(&mut self, chunk: &[u8]) {
        (**self).computed(chunk)
    }
}

#[inline(always)]
fn visit_chunks<'a>(
    bytes: &'a [u8],
    chunks: &[LenPrefixedChunk],
    mut visitor: impl ChunkVisitor<'a>,
) {
    let mut start = 0;
    for chunk in chunks.iter() {
        // Write the previous chunk
        visitor.borrowed(&bytes[start..chunk.start]);

        // Write the current varint
        if let Some(varint) = chunk.varint {
            visitor.computed(VarInt::uint64(varint).fill_bytes(&mut [0; 10]));
        }

        start = chunk.start;
    }

    // Write the trailing portion of the buffer
    visitor.borrowed(&bytes[start..]);
}

pub struct ProtoBufCursor {
    bytes: Box<[u8]>,
    chunks: IterBox<LenPrefixedChunk>,
    // Typically called in loops, so it's good to have this
    // value readily available
    remaining: usize,
    next: NextChunk,
    current: CurrentChunk,
}

struct IterBox<T> {
    items: Box<[T]>,
    head: usize,
}

impl<T> Iterator for IterBox<T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.head).copied();
        self.head += 1;

        item
    }
}

impl ProtoBufCursor {
    fn new(bytes: Box<[u8]>, chunks: Box<[LenPrefixedChunk]>) -> Self {
        let remaining = len(&bytes, &chunks);

        let mut chunks = IterBox {
            items: chunks,
            head: 0,
        };

        let mut cursor = if let Some(chunk) = chunks.next() {
            ProtoBufCursor {
                bytes,
                chunks,
                remaining,
                next: NextChunk::Chunk(0, chunk),
                current: CurrentChunk::empty(),
            }
        } else {
            ProtoBufCursor {
                bytes,
                chunks,
                remaining,
                next: NextChunk::Trailing(0),
                current: CurrentChunk::empty(),
            }
        };

        cursor.move_next();
        cursor
    }

    fn move_next(&mut self) -> bool {
        match self.next {
            NextChunk::Chunk(from, chunk) => {
                let to = chunk.start;

                self.current = CurrentChunk::bytes(from..to);

                if let Some(varint) = chunk.varint {
                    self.next = NextChunk::VarInt(to, varint);
                } else if let Some(next) = self.chunks.next() {
                    self.next = NextChunk::Chunk(to, next);
                } else {
                    self.next = NextChunk::Trailing(to);
                }

                true
            }
            NextChunk::VarInt(to, varint) => {
                self.current = CurrentChunk::varint(VarInt::uint64(varint));

                if let Some(next) = self.chunks.next() {
                    self.next = NextChunk::Chunk(to, next);
                } else {
                    self.next = NextChunk::Trailing(to);
                }

                true
            }
            NextChunk::Trailing(from) => {
                self.current = CurrentChunk::bytes(from..self.bytes.len());

                self.next = NextChunk::Done;

                true
            }
            NextChunk::Done => {
                self.current = CurrentChunk::Empty;

                false
            }
        }
    }

    pub fn chunk(&self) -> &[u8] {
        self.current.as_slice(&self.bytes)
    }

    pub fn advance(&mut self, mut cnt: usize) {
        self.remaining = self.remaining.saturating_sub(cnt);

        loop {
            let (cnt_remaining, chunk_remaining) = self.current.advance(cnt);
            cnt = cnt_remaining;

            let has_next = if chunk_remaining == 0 {
                self.move_next()
            } else {
                true
            };

            if cnt == 0 {
                return;
            } else if has_next {
                continue;
            } else {
                panic!("attempt to advance past the end of the buffer");
            }
        }
    }

    pub fn remaining(&self) -> usize {
        self.remaining
    }

    pub fn copy_to_vec(&mut self, v: &mut Vec<u8>) {
        v.reserve(self.remaining());

        while self.remaining() > 0 {
            let chunk = self.chunk();
            v.extend_from_slice(chunk);
            self.advance(chunk.len());
        }
    }
}

enum NextChunk {
    Chunk(usize, LenPrefixedChunk),
    VarInt(usize, u64),
    Trailing(usize),
    Done,
}

enum CurrentChunk {
    Empty,
    Bytes {
        remaining: Range<usize>,
    },
    VarInt {
        buf: [u8; 10],
        remaining: Range<usize>,
    },
}

impl CurrentChunk {
    fn empty() -> Self {
        CurrentChunk::Empty
    }

    fn bytes(range: Range<usize>) -> Self {
        CurrentChunk::Bytes { remaining: range }
    }

    fn varint(varint: VarInt) -> Self {
        let mut buf = [0; 10];
        let len = varint.fill_bytes(&mut buf).len();

        CurrentChunk::VarInt {
            buf,
            remaining: 0..len,
        }
    }

    fn advance(&mut self, cnt: usize) -> (usize, usize) {
        let remaining = match self {
            CurrentChunk::Bytes { remaining } => remaining,
            CurrentChunk::VarInt { remaining, .. } => remaining,
            CurrentChunk::Empty => return (cnt, 0),
        };

        let from = remaining.start;
        remaining.start = cmp::min(remaining.start + cnt, remaining.end);

        let cnt_remaining = cnt.saturating_sub(remaining.start - from);
        let chunk_remaining = remaining.len();

        (cnt_remaining, chunk_remaining)
    }

    fn as_slice<'a>(&'a self, bytes: &'a [u8]) -> &'a [u8] {
        match self {
            CurrentChunk::Bytes { remaining } => &bytes[remaining.clone()],
            CurrentChunk::VarInt { buf, remaining } => &buf[remaining.clone()],
            CurrentChunk::Empty => &[],
        }
    }
}

#[cfg(feature = "bytes")]
mod bytes_support {
    use super::*;

    impl bytes::Buf for ProtoBufCursor {
        fn chunk(&self) -> &[u8] {
            self.chunk()
        }

        fn remaining(&self) -> usize {
            self.remaining()
        }

        fn advance(&mut self, cnt: usize) {
            self.advance(cnt);
        }
    }
}
