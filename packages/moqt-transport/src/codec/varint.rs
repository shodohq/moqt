use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Variable-Length Integer Encoding
///
/// https://datatracker.ietf.org/doc/html/rfc9000#name-variable-length-integer-enc
pub struct VarInt;

impl Encoder<u64> for VarInt {
    type Error = crate::error::Error;

    fn encode(&mut self, item: u64, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item < (1 << 6) {
            dst.put_u8(item as u8);
        } else if item < (1 << 14) {
            dst.put_u8(0x40 | ((item >> 8) as u8));
            dst.put_u8(item as u8);
        } else if item < (1 << 30) {
            dst.put_u8(0x80 | ((item >> 24) as u8));
            dst.put_u8(((item >> 16) & 0xff) as u8);
            dst.put_u8(((item >> 8) & 0xff) as u8);
            dst.put_u8(item as u8);
        } else if item < (1 << 62) {
            dst.put_u8(0xC0 | ((item >> 56) as u8));
            dst.put_u8(((item >> 48) & 0xff) as u8);
            dst.put_u8(((item >> 40) & 0xff) as u8);
            dst.put_u8(((item >> 32) & 0xff) as u8);
            dst.put_u8(((item >> 24) & 0xff) as u8);
            dst.put_u8(((item >> 16) & 0xff) as u8);
            dst.put_u8(((item >> 8) & 0xff) as u8);
            dst.put_u8(item as u8);
        } else {
            return Err(crate::error::Error::VarIntRange);
        }
        Ok(())
    }
}

impl Decoder for VarInt {
    type Item = u64;
    type Error = crate::error::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let first = match src.first() {
            Some(v) => *v,
            None => return Ok(None),
        };

        let prefix = first >> 6;
        let len = 1usize << prefix;

        if src.len() < len {
            return Ok(None);
        }

        let value = match len {
            1 => (first & 0x3f) as u64,
            2 => {
                let b1 = src[1] as u64;
                ((first as u64 & 0x3f) << 8) | b1
            }
            4 => {
                ((first as u64 & 0x3f) << 24)
                    | ((src[1] as u64) << 16)
                    | ((src[2] as u64) << 8)
                    | src[3] as u64
            }
            8 => {
                ((first as u64 & 0x3f) << 56)
                    | ((src[1] as u64) << 48)
                    | ((src[2] as u64) << 40)
                    | ((src[3] as u64) << 32)
                    | ((src[4] as u64) << 24)
                    | ((src[5] as u64) << 16)
                    | ((src[6] as u64) << 8)
                    | src[7] as u64
            }
            _ => unreachable!(),
        };

        let _ = src.split_to(len);
        Ok(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn encode_examples() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (63, &[0x3f]),
            (64, &[0x40, 0x40]),
            (16383, &[0x7f, 0xff]),
            (16384, &[0x80, 0x00, 0x40, 0x00]),
            (1073741823, &[0xbf, 0xff, 0xff, 0xff]),
            (
                1073741824,
                &[0xc0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00],
            ),
        ];

        for (value, expected) in cases {
            let mut buf = BytesMut::new();
            VarInt.encode(*value, &mut buf).unwrap();
            assert_eq!(buf.as_ref(), *expected);
        }
    }

    #[test]
    fn decode_examples() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (63, &[0x3f]),
            (64, &[0x40, 0x40]),
            (16383, &[0x7f, 0xff]),
            (16384, &[0x80, 0x00, 0x40, 0x00]),
            (1073741823, &[0xbf, 0xff, 0xff, 0xff]),
            (
                1073741824,
                &[0xc0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00],
            ),
        ];

        for (expected, bytes) in cases {
            let mut buf = BytesMut::from(*bytes);
            let value = VarInt.decode(&mut buf).unwrap().unwrap();
            assert_eq!(value, *expected);
            assert!(buf.is_empty());
        }
    }

    #[test]
    fn decode_incomplete_returns_none() {
        let mut buf = BytesMut::from(&b"\x40"[..]);
        assert!(VarInt.decode(&mut buf).unwrap().is_none());
        assert_eq!(buf.len(), 1);
    }
}
