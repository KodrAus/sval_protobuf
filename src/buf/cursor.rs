use super::{visitor::len, LenPrefixedChunk};
use crate::raw::VarInt;

use alloc::{boxed::Box, vec::Vec};
use core::{cmp, ops::Range};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_empty() {
        todo!()
    }

    #[test]
    fn cursor_invalid() {
        todo!()
    }

    #[test]
    fn advance_zero() {
        todo!()
    }

    #[test]
    fn advance_through_chunk() {
        todo!()
    }
}
