use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

/// Representation of the REQUESTS_BLOCKED message body.
///
/// This message carries the maximum request ID advertised by the peer when the
/// sender became blocked.  It consists of a single variable-length integer
/// value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RequestsBlocked {
    /// The maximum request ID value that caused the sender to become blocked.
    pub maximum_request_id: u64,
}

impl RequestsBlocked {
    /// Encode the REQUESTS_BLOCKED message body into the provided buffer.
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;
        vi.encode(self.maximum_request_id, buf)?;
        Ok(())
    }

    /// Decode a REQUESTS_BLOCKED message body from the provided buffer.
    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;
        let maximum_request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "maximum request id"))?;

        Ok(RequestsBlocked { maximum_request_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = RequestsBlocked {
            maximum_request_id: 42,
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = RequestsBlocked::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
