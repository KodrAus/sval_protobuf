use super::{visit::len, LenPrefixedChunk};
use crate::raw::VarInt;

use alloc::{boxed::Box, vec::Vec};
use core::{cmp, ops::Range};

/**
A reader over an encoded protobuf message that offers a similar API to the `bytes` crate.
*/
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
    pub(super) fn new(bytes: Box<[u8]>, chunks: Box<[LenPrefixedChunk]>) -> Self {
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

    /**
    Get the next contiguous chunk of data in the message.

    The size of chunks will depend on how the message was originally encoded.
    Messages produced from streams of values of unknown size will produce smaller chunks.
    */
    #[inline]
    pub fn chunk(&self) -> &[u8] {
        self.current.as_slice(&self.bytes)
    }

    /**
    Advance the cursor by `cnt`.

    This method will panic if `cnt` is greater than [`Self::remaining`].
    */
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

    /**
    The number of bytes left to read.
    */
    #[inline]
    pub fn remaining(&self) -> usize {
        self.remaining
    }

    /**
    Copy all remaining bytes into a contiguous vec.
    */
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
    #[inline]
    fn empty() -> Self {
        CurrentChunk::Empty
    }

    #[inline]
    fn bytes(range: Range<usize>) -> Self {
        CurrentChunk::Bytes { remaining: range }
    }

    #[inline]
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

        // We may advance past the end of this chunk and into the next one
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
        #[inline]
        fn chunk(&self) -> &[u8] {
            self.chunk()
        }

        #[inline]
        fn remaining(&self) -> usize {
            self.remaining()
        }

        #[inline]
        fn advance(&mut self, cnt: usize) {
            self.advance(cnt);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buf::ProtoBufMut;

    use super::*;

    #[test]
    fn read_cursor_chunked() {
        let mut buf = ProtoBufMut::new(());

        buf.push(b"abc");
        buf.begin_len(());
        buf.push(b"def");
        buf.end_len();
        buf.push(b"ghi");
        buf.begin_len(());
        buf.push(b"jkl");
        buf.end_len();
        buf.push(b"mno");

        let buf = buf.freeze();

        let expected = buf.to_vec().into_owned();

        let mut cursor = buf.into_cursor();

        assert_ne!(0, cursor.chunks.items.len());

        let mut read = Vec::new();
        cursor.copy_to_vec(&mut read);

        assert_eq!(expected, read);
    }

    #[test]
    fn read_cursor_contiguous() {
        let mut buf = ProtoBufMut::new(());

        buf.push(b"abc");
        buf.push(b"def");
        buf.push(b"ghi");
        buf.push(b"jkl");
        buf.push(b"mno");

        let buf = buf.freeze();

        let expected = buf.to_vec().into_owned();

        let mut cursor = buf.into_cursor();

        assert_eq!(0, cursor.chunks.items.len());

        let mut read = Vec::new();
        cursor.copy_to_vec(&mut read);

        assert_eq!(expected, read);
    }

    #[test]
    fn read_cursor_empty() {
        let buf = ProtoBufMut::new(()).freeze();

        let mut read = Vec::new();
        buf.into_cursor().copy_to_vec(&mut read);

        assert_eq!(0, read.len());
    }

    #[test]
    fn read_cursor_invalid() {
        let mut buf = ProtoBufMut::new(());

        buf.push(b"ab");
        buf.begin_len(());
        buf.push(b"cde");
        buf.end_len();
        buf.push(b"efg");
        buf.begin_len(());
        buf.push(b"h");
        buf.begin_len(());
        buf.end_len();

        let mut read = Vec::new();
        buf.freeze().into_cursor().copy_to_vec(&mut read);

        // Just ensure we don't loop or panic reading an incomplete buffer
        assert_ne!(0, read.len());
    }

    #[test]
    fn advance_zero() {
        let mut buf = ProtoBufMut::new(());

        buf.push(b"abc");
        buf.begin_len(());
        buf.push(b"def");
        buf.end_len();
        buf.push(b"ghi");
        buf.begin_len(());
        buf.push(b"jkl");
        buf.end_len();
        buf.push(b"mno");

        let mut cursor = buf.freeze().into_cursor();

        while cursor.remaining() > 0 {
            cursor.advance(1);

            let chunk = cursor.chunk().to_vec();
            cursor.advance(0);

            assert_eq!(chunk, cursor.chunk());
        }
    }

    #[test]
    fn advance_through_chunk() {
        let mut buf = ProtoBufMut::new(());

        buf.push(b"abc");
        buf.begin_len(());
        buf.push(b"def");
        buf.end_len();
        buf.push(b"ghi");
        buf.begin_len(());
        buf.push(b"jkl");
        buf.end_len();
        buf.push(b"mno");

        let mut cursor = buf.freeze().into_cursor();

        let remaining = cursor.remaining();

        cursor.advance(remaining - 3);

        let mut read = Vec::new();
        cursor.copy_to_vec(&mut read);

        assert_eq!(3, read.len());
    }

    #[test]
    #[should_panic]
    fn err_advance_past_end() {
        let mut buf = ProtoBufMut::new(());

        buf.push(b"abc");

        let mut cursor = buf.freeze().into_cursor();

        let remaining = cursor.remaining();

        cursor.advance(remaining + 1);
    }
}
