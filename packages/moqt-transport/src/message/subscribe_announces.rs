use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::Parameter;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscribeAnnounces {
    pub request_id: u64,
    pub track_namespace_prefix: Vec<String>,
    pub parameters: Vec<Parameter>,
}

impl SubscribeAnnounces {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        if self.track_namespace_prefix.is_empty() || self.track_namespace_prefix.len() > 32 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid prefix length").into());
        }

        vi.encode(self.request_id, buf)?;

        vi.encode(self.track_namespace_prefix.len() as u64, buf)?;
        for part in &self.track_namespace_prefix {
            vi.encode(part.len() as u64, buf)?;
            buf.put_slice(part.as_bytes());
        }

        vi.encode(self.parameters.len() as u64, buf)?;
        for p in &self.parameters {
            vi.encode(p.parameter_type, buf)?;
            vi.encode(p.value.len() as u64, buf)?;
            buf.put_slice(&p.value);
        }

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;

        let prefix_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "prefix len"))?
            as usize;

        if prefix_len == 0 || prefix_len > 32 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid prefix length").into());
        }

        let mut track_namespace_prefix = Vec::with_capacity(prefix_len);
        for _ in 0..prefix_len {
            let part_len = vi
                .decode(buf)?
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "part len"))?
                as usize;
            if buf.len() < part_len {
                return Err(IoError::new(ErrorKind::UnexpectedEof, "part").into());
            }
            let bytes = buf.split_to(part_len);
            let part = String::from_utf8(bytes.to_vec())
                .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;
            track_namespace_prefix.push(part);
        }

        let params_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameters len"))?
            as usize;

        let mut parameters = Vec::with_capacity(params_len);
        for _ in 0..params_len {
            let ty = vi
                .decode(buf)?
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameter type"))?;
            let len = vi
                .decode(buf)?
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameter len"))?
                as usize;
            if buf.len() < len {
                return Err(IoError::new(ErrorKind::UnexpectedEof, "parameter value").into());
            }
            let value = buf.split_to(len).to_vec();
            parameters.push(Parameter {
                parameter_type: ty,
                value,
            });
        }

        Ok(SubscribeAnnounces {
            request_id,
            track_namespace_prefix,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = SubscribeAnnounces {
            request_id: 1,
            track_namespace_prefix: vec!["example.com".into(), "meeting=123".into()],
            parameters: vec![Parameter {
                parameter_type: 1,
                value: vec![42],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeAnnounces::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match SubscribeAnnounces::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn decode_fails_on_invalid_prefix_len() {
        let mut buf = BytesMut::new();
        let mut vi = crate::coding::VarInt;
        vi.encode(1, &mut buf).unwrap(); // request_id
        vi.encode(0, &mut buf).unwrap(); // invalid prefix length
        vi.encode(0, &mut buf).unwrap(); // parameters len

        assert!(SubscribeAnnounces::decode(&mut buf).is_err());
    }
}
