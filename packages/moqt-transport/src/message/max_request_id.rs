use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::codec::{Decode, Encode};

/// MAX_REQUEST_ID
///
/// https://datatracker.ietf.org/doc/html/draft-ietf-moq-transport-12#name-max_request_id-2
///
/// An endpoint sends a MAX_REQUEST_ID message to increase the number of
/// requests the peer can send within a session.
/// The Maximum Request ID MUST only increase within a session, and
/// receipt of a MAX_REQUEST_ID message with an equal or smaller Request
/// ID value is a 'Protocol Violation'.
///
/// ```text
/// MAX_REQUEST_ID Message {
///   Type (i) = 0x15,
///   Length (16),
///   Request ID (i),
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MaxRequestId {
    pub request_id: u64,
}

impl Encode for MaxRequestId {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;
        vi.encode(self.request_id, buf)?;
        Ok(())
    }
}

impl Decode for MaxRequestId {
    fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;
        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;

        Ok(MaxRequestId { request_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = MaxRequestId {
            request_id: 0xabcdef,
        }; // some arbitrary id

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = MaxRequestId::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_large_id() {
        // use a value that requires the largest varint encoding
        let msg = MaxRequestId {
            request_id: 0x1234_5678_9abc_def0,
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = MaxRequestId::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_fails_on_too_large_id() {
        // values >= 2^62 cannot be encoded as a varint
        let msg = MaxRequestId {
            request_id: 1u64 << 62,
        };

        let mut buf = BytesMut::new();
        match msg.encode(&mut buf) {
            Err(crate::error::Error::VarIntRange) => {}
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match MaxRequestId::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn decode_incomplete_varint() {
        let mut buf = BytesMut::from(&b"\x40"[..]);
        match MaxRequestId::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
