use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Representation of a GOAWAY message body.
///
/// The message optionally carries a new session URI.  When the length is
/// `0`, the current session URI is reused.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Goaway {
    /// Optional URI for the new session.
    pub new_session_uri: Option<String>,
}

impl Goaway {
    /// Encode the GOAWAY message body into the provided buffer.
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        if let Some(uri) = &self.new_session_uri {
            let bytes = uri.as_bytes();
            if bytes.len() > 8192 {
                return Err(IoError::new(ErrorKind::InvalidData, "uri too long").into());
            }
            vi.encode(bytes.len() as u64, buf)?;
            buf.put_slice(bytes);
        } else {
            vi.encode(0, buf)?;
        }

        Ok(())
    }

    /// Decode a GOAWAY message body from the provided buffer.
    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        let len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "uri length"))?
            as usize;

        if len > 8192 {
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
    fn decode_fails_on_oversized_uri() {
        let uri_len = 8193u64; // larger than allowed
        let mut buf = BytesMut::new();
        crate::codec::VarInt.encode(uri_len, &mut buf).unwrap();
        buf.resize(buf.len() + uri_len as usize, 0);

        assert!(Goaway::decode(&mut buf).is_err());
    }
}
