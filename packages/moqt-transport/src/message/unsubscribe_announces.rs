use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnsubscribeAnnounces {
    pub track_namespace: u64,
    pub track_name_prefix: String,
}

impl UnsubscribeAnnounces {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::coding::VarInt;

        vi.encode(self.track_namespace, buf)?;

        vi.encode(self.track_name_prefix.len() as u64, buf)?;
        buf.put_slice(self.track_name_prefix.as_bytes());

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        let track_namespace = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track namespace"))?;

        let prefix_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track name prefix len"))?
            as usize;

        if buf.len() < prefix_len {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "track name prefix").into());
        }
        let prefix_bytes = buf.split_to(prefix_len);
        let track_name_prefix = String::from_utf8(prefix_bytes.to_vec())
            .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;

        Ok(UnsubscribeAnnounces {
            track_namespace,
            track_name_prefix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = UnsubscribeAnnounces {
            track_namespace: 1,
            track_name_prefix: "video".into(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = UnsubscribeAnnounces::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match UnsubscribeAnnounces::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
