use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unsubscribe {
    pub request_id: u64,
}

impl Unsubscribe {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;
        vi.encode(self.request_id, buf)?;
        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;
        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;

        Ok(Unsubscribe { request_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = Unsubscribe { request_id: 42 };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Unsubscribe::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match Unsubscribe::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
