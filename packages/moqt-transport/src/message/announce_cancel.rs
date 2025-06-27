use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Representation of an ANNOUNCE_CANCEL message body.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnnounceCancel {
    /// Track namespace for which announcements are cancelled.
    pub track_namespace: u64,
    /// Error code describing the reason for cancellation.
    pub error_code: u64,
    /// Human readable reason for the cancellation.
    pub error_reason: String,
}

impl AnnounceCancel {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.track_namespace, buf)?;
        vi.encode(self.error_code, buf)?;

        let reason_bytes = self.error_reason.as_bytes();
        vi.encode(reason_bytes.len() as u64, buf)?;
        buf.put_slice(reason_bytes);

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        let track_namespace = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track namespace"))?;
        let error_code = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "error code"))?;

        let reason_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "reason length"))?
            as usize;
        if buf.len() < reason_len {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "reason").into());
        }
        let value = buf.split_to(reason_len);
        let error_reason = String::from_utf8(value.to_vec())
            .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;

        Ok(AnnounceCancel {
            track_namespace,
            error_code,
            error_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_with_reason() {
        let msg = AnnounceCancel {
            track_namespace: 42,
            error_code: 3,
            error_reason: "going away".into(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = AnnounceCancel::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_empty_reason() {
        let msg = AnnounceCancel {
            track_namespace: 1,
            error_code: 0x1,
            error_reason: String::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = AnnounceCancel::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
