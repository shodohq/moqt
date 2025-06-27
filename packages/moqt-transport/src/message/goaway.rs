use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// GOAWAY
///
/// https://datatracker.ietf.org/doc/html/draft-ietf-moq-transport-12#name-goaway
///
/// An endpoint sends a GOAWAY message to inform the peer it intends to
/// close the session soon.  Servers can use GOAWAY to initiate session
/// migration (Section 3.5) with an optional URI.
/// The GOAWAY message does not impact subscription state.  A subscriber
/// SHOULD individually UNSUBSCRIBE for each existing subscription, while
/// a publisher MAY reject new requests while in the draining state.
/// Upon receiving a GOAWAY, an endpoint SHOULD NOT initiate new requests
/// to the peer including SUBSCRIBE, PUBLISH, FETCH, ANNOUNCE and
/// SUBSCRIBE_ANNOUNCE.
/// The endpoint MUST terminate the session with a Protocol Violation
/// (Section 3.4) if it receives multiple GOAWAY messages.
///
/// ```text
/// GOAWAY Message {
///   Type (i) = 0x10,
///   Length (16),
///   New Session URI Length (i),
///   New Session URI (..),
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Goaway {
    pub new_session_uri: Option<String>,
}

impl Goaway {
    const MAX_URI_LENGTH: usize = 8_192;

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::coding::VarInt;

        // New Session URI
        if let Some(uri) = &self.new_session_uri {
            let bytes = uri.as_bytes();
            if bytes.len() > Self::MAX_URI_LENGTH {
                use std::io::{Error as IoError, ErrorKind};
                return Err(IoError::new(ErrorKind::InvalidData, "uri too long").into());
            }
            vi.encode(bytes.len() as u64, buf)?;
            buf.put_slice(bytes);
        } else {
            vi.encode(0, buf)?;
        }

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        // New Session URI
        let len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "uri length"))?
            as usize;
        if len > Self::MAX_URI_LENGTH {
            return Err(IoError::new(ErrorKind::InvalidData, "uri too long").into());
        }
        if buf.len() < len {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "uri").into());
        }
        let value = buf.split_to(len);
        let new_session_uri = if len == 0 {
            None
        } else {
            Some(
                String::from_utf8(value.to_vec())
                    .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?,
            )
        };

        Ok(Goaway { new_session_uri })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::codec::Encoder;

    #[test]
    fn encode_decode_roundtrip_with_uri() {
        let msg = Goaway {
            new_session_uri: Some("https://example.com/moq".to_string()),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Goaway::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_without_uri() {
        let msg = Goaway {
            new_session_uri: None,
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Goaway::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_fails_on_long_uri() {
        let msg = Goaway {
            new_session_uri: Some("a".repeat(Goaway::MAX_URI_LENGTH + 1)),
        };

        let mut buf = BytesMut::new();
        assert!(msg.encode(&mut buf).is_err());
    }

    #[test]
    fn decode_fails_on_long_uri() {
        let mut buf = BytesMut::new();
        let mut vi = crate::coding::VarInt;
        vi.encode((Goaway::MAX_URI_LENGTH + 1) as u64, &mut buf).unwrap();
        buf.extend(std::iter::repeat(b'a').take(Goaway::MAX_URI_LENGTH + 1));

        assert!(Goaway::decode(&mut buf).is_err());
    }
}
