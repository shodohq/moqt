use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unannounce {
    pub track_namespace: u64,
}

impl Unannounce {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;
        vi.encode(self.track_namespace, buf)?;
        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;
        let track_namespace = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track namespace"))?;

        Ok(Unannounce { track_namespace })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = Unannounce { track_namespace: 42 };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Unannounce::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match Unannounce::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
