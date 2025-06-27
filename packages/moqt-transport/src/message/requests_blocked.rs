use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

/// REQUESTS_BLOCKED
///
/// https://datatracker.ietf.org/doc/html/draft-ietf-moq-transport-12#name-requests_blocked
///
/// The REQUESTS_BLOCKED message is sent when an endpoint would like to
/// send a new request, but cannot because the Request ID would exceed
/// the Maximum Request ID value sent by the peer.  The endpoint SHOULD
/// send only one REQUESTS_BLOCKED for a given Maximum Request ID.
/// An endpoint MAY send a MAX_REQUEST_ID upon receipt of
/// REQUESTS_BLOCKED, but it MUST NOT rely on REQUESTS_BLOCKED to trigger
/// sending a MAX_REQUEST_ID, because sending REQUESTS_BLOCKED is not
/// required.
///
/// ```text
/// REQUESTS_BLOCKED Message {
///   Type (i) = 0x1A,
///   Length (16),
///   Maximum Request ID (i),
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RequestsBlocked {
    pub maximum_request_id: u64,
}

impl RequestsBlocked {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;
        vi.encode(self.maximum_request_id, buf)?;
        Ok(())
    }

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

    #[test]
    fn encode_decode_roundtrip_large_id() {
        // use a value that requires the largest varint encoding
        let msg = RequestsBlocked {
            maximum_request_id: 0x1234_5678_9abc_def0,
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = RequestsBlocked::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match RequestsBlocked::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn decode_incomplete_varint() {
        let mut buf = BytesMut::from(&b"\x40"[..]);
        match RequestsBlocked::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn encode_fails_on_too_large_id() {
        // values >= 2^62 cannot be encoded as a varint
        let msg = RequestsBlocked {
            maximum_request_id: 1u64 << 62,
        };

        let mut buf = BytesMut::new();
        match msg.encode(&mut buf) {
            Err(crate::error::Error::VarIntRange) => {}
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
