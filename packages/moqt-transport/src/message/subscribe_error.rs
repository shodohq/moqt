use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Representation of a SUBSCRIBE_ERROR message body.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscribeError {
    /// The request ID of the SUBSCRIBE message this is replying to.
    pub request_id: u64,
    /// The error code for the failure.
    pub error_code: u64,
    /// Human readable reason for the error.
    pub error_reason: String,
}

impl SubscribeError {
    /// Encode the message body into the provided buffer.
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.request_id, buf)?;
        vi.encode(self.error_code, buf)?;

        let reason_bytes = self.error_reason.as_bytes();
        vi.encode(reason_bytes.len() as u64, buf)?;
        buf.put_slice(reason_bytes);

        Ok(())
    }

    /// Decode the message body from the provided buffer.
    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;
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

        Ok(SubscribeError {
            request_id,
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
        let msg = SubscribeError {
            request_id: 42,
            error_code: 0x4,
            error_reason: "track missing".into(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeError::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_empty_reason() {
        let msg = SubscribeError {
            request_id: 1,
            error_code: 0x1,
            error_reason: String::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeError::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
