/*!
Raw protobuf wire format.

The protobuf wire format is described in detail in [the docs](https://protobuf.dev/programming-guides/encoding/).
It's a length-prefixed format that makes extensive use of variable-length integers.
*/

use core::mem;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VarInt(u64);

impl VarInt {
    pub const WIRE_TYPE: WireType = WireType::VarInt;

    #[inline(always)]
    pub fn uint64(v: u64) -> Self {
        VarInt(v)
    }

    #[inline(always)]
    pub fn sint64(v: i64) -> Self {
        // Re-interpret the bits of `v` without truncating
        // For negative values this will be interpreted as some large number
        VarInt(v as u64)
    }

    #[inline(always)]
    pub fn sint64z(v: i64) -> Self {
        VarInt((v << 1) as u64 ^ (v >> 63) as u64)
    }

    #[inline(always)]
    pub fn bool(v: bool) -> Self {
        VarInt(if v { 1 } else { 0 })
    }

    #[inline(always)]
    pub fn enum32(v: i32) -> Self {
        VarInt(v as u64)
    }

    #[inline(always)]
    pub fn field(field_number: u64, wire_type: WireType) -> Self {
        VarInt((field_number << 3) | (wire_type as u64))
    }

    #[inline(always)]
    pub fn fill_bytes<'a>(&self, buf: &'a mut [u8; 10]) -> &'a [u8] {
        let mut v = self.0;
        let mut i = 0;

        while v >= 0b1000_0000 {
            buf[i] = ((v & 0b0111_1111) | 0b1000_0000) as u8;
            i += 1;

            v >>= 7;
        }

        buf[i] = v as u8;

        &buf[..i + 1]
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        // From:
        // [1]: https://github.com/tokio-rs/prost/blob/97cd4e29c46f1cac4d27428c759b6bc807c37201/src/encoding.rs#L261-L264
        // [2]: https://github.com/google/protobuf/blob/3.3.x/src/google/protobuf/io/coded_stream.h#L1301-L1309
        ((((self.0 | 1).leading_zeros() ^ 63) * 9 + 73) / 64) as usize
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I32(u32);

impl I32 {
    pub const WIRE_TYPE: WireType = WireType::I32;

    #[inline(always)]
    pub fn float(v: f32) -> Self {
        I32(v.to_bits())
    }

    #[inline(always)]
    pub fn fixed32(v: u32) -> Self {
        I32(v)
    }

    #[inline(always)]
    pub fn sfixed32(v: i32) -> Self {
        I32(v as u32)
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I64(u64);

impl I64 {
    pub const WIRE_TYPE: WireType = WireType::I64;

    #[inline(always)]
    pub fn double(v: f64) -> Self {
        I64(v.to_bits())
    }

    #[inline(always)]
    pub fn fixed64(v: u64) -> Self {
        I64(v)
    }

    #[inline(always)]
    pub fn sfixed64(v: i64) -> Self {
        I64(v as u64)
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WireType {
    VarInt = 0,
    I64 = 1,
    Len = 2,
    I32 = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_uint64_len() {
        for n in (0u64..(u16::MAX as u64)).into_iter().chain(
            [
                u32::MAX as u64 - 1,
                u32::MAX as u64,
                u32::MAX as u64 + 1,
                u64::MAX,
            ]
            .into_iter(),
        ) {
            let v = VarInt::uint64(n);

            assert_eq!(v.len(), v.fill_bytes(&mut [0; 10]).len(), "{}", n);
        }
    }

    #[test]
    fn encode_varint_uint64() {
        for (case, expected) in [
            (0, &[0u8] as &[u8]),
            (1, &[1u8] as &[u8]),
            (255, &[255u8, 1u8] as &[u8]),
            (
                u64::MAX,
                &[
                    255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 1u8,
                ] as &[u8],
            ),
        ] {
            let varint = VarInt::uint64(case);

            assert_eq!(expected, varint.fill_bytes(&mut [0; 10]), "{}", case);

            assert_eq!(
                varint.len(),
                varint.fill_bytes(&mut [0; 10]).len(),
                "{}",
                case
            );
        }
    }

    #[test]
    fn encode_varint_sint64() {
        for (case, expected) in [
            (0, &[0u8] as &[u8]),
            (1, &[1u8] as &[u8]),
            (
                -1,
                &[
                    255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 1u8,
                ] as &[u8],
            ),
            (
                i64::MIN,
                &[
                    128u8, 128u8, 128u8, 128u8, 128u8, 128u8, 128u8, 128u8, 128u8,
                ] as &[u8],
            ),
            (
                i64::MAX,
                &[
                    255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 127u8,
                ] as &[u8],
            ),
        ] {
            assert_eq!(
                expected,
                VarInt::sint64(case).fill_bytes(&mut [0; 10]),
                "{}",
                case
            );
        }
    }

    #[test]
    fn encode_varint_sint64z() {
        for (case, expected) in [
            (0, &[0u8] as &[u8]),
            (1, &[2u8] as &[u8]),
            (-1, &[1u8] as &[u8]),
            (
                i64::MIN,
                &[
                    255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 1u8,
                ] as &[u8],
            ),
            (
                i64::MAX,
                &[
                    254u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 1u8,
                ] as &[u8],
            ),
        ] {
            assert_eq!(
                expected,
                VarInt::sint64z(case).fill_bytes(&mut [0; 10]),
                "{}",
                case
            );
        }
    }
}
