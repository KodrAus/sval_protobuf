use crate::raw::VarInt;
use alloc::{borrow::Cow, vec::Vec};

use super::LenPrefixedChunk;

trait Visitor<'a> {
    fn borrowed(&mut self, chunk: &'a [u8]) {
        self.computed(chunk);
    }

    fn computed(&mut self, chunk: &[u8]);
}

impl<'a, 'b, V: Visitor<'a> + ?Sized> Visitor<'a> for &'b mut V {
    fn borrowed(&mut self, chunk: &'a [u8]) {
        (**self).borrowed(chunk)
    }

    fn computed(&mut self, chunk: &[u8]) {
        (**self).computed(chunk)
    }
}

#[inline(always)]
fn visit_chunks<'a>(bytes: &'a [u8], chunks: &[LenPrefixedChunk], mut visitor: impl Visitor<'a>) {
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

pub(super) fn len(bytes: &[u8], chunks: &[LenPrefixedChunk]) -> usize {
    bytes.len()
        + chunks
            .iter()
            .filter_map(|chunk| chunk.varint)
            .map(|varint| VarInt::uint64(varint).len())
            .sum::<usize>()
}

pub(super) fn to_stream<'a>(
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

        impl<'sval, S: sval::Stream<'sval>> Visitor<'sval> for StreamVisitor<S> {
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
pub(super) fn to_vec<'a>(bytes: &'a [u8], chunks: &[LenPrefixedChunk]) -> Cow<'a, [u8]> {
    if chunks.len() == 0 {
        return Cow::Borrowed(&bytes);
    }

    struct BufVisitor(Vec<u8>);

    impl<'a> Visitor<'a> for BufVisitor {
        fn computed(&mut self, chunk: &[u8]) {
            self.0.extend_from_slice(chunk);
        }
    }

    let mut visitor = BufVisitor(Vec::with_capacity(len(bytes, chunks)));
    visit_chunks(bytes, chunks, &mut visitor);

    debug_assert_eq!(
        len(bytes, chunks),
        visitor.0.len(),
        "{:?} + {:?}",
        bytes.len(),
        chunks
    );

    Cow::Owned(visitor.0)
}
