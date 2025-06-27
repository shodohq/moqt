use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::Parameter;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TrackStatusRequest {
    pub request_id: u64,
    pub track_namespace: u64,
    pub track_name: String,
    pub parameters: Vec<Parameter>,
}

impl TrackStatusRequest {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::coding::VarInt;

        vi.encode(self.request_id, buf)?;
        vi.encode(self.track_namespace, buf)?;

        vi.encode(self.track_name.len() as u64, buf)?;
        buf.put_slice(self.track_name.as_bytes());

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

        let track_namespace = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track namespace"))?;

        let name_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track name len"))?
            as usize;

        if buf.len() < name_len {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "track name").into());
        }
        let name_bytes = buf.split_to(name_len);
        let track_name = String::from_utf8(name_bytes.to_vec())
            .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;

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

        Ok(TrackStatusRequest {
            request_id,
            track_namespace,
            track_name,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = TrackStatusRequest {
            request_id: 1,
            track_namespace: 2,
            track_name: "video".into(),
            parameters: vec![Parameter {
                parameter_type: 4,
                value: vec![7, 8],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = TrackStatusRequest::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match TrackStatusRequest::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
