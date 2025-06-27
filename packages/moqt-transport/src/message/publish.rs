use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Publish {
    pub request_id: u64,
    pub track_namespace: u64,
    pub track_name: String,
    pub track_alias: u64,
    pub group_order: u8,
    pub content_exists: u8,
    pub largest: Option<Location>,
    pub forward: u8,
    pub parameters: Vec<Parameter>,
}

impl Publish {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        vi.encode(self.request_id, buf)?;
        vi.encode(self.track_namespace, buf)?;

        vi.encode(self.track_name.len() as u64, buf)?;
        buf.put_slice(self.track_name.as_bytes());

        vi.encode(self.track_alias, buf)?;

        if self.group_order == 0 || self.group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }
        buf.put_u8(self.group_order);

        if self.content_exists != 0 && self.content_exists != 1 {
            return Err(
                IoError::new(ErrorKind::InvalidData, "invalid content exists value").into(),
            );
        }
        buf.put_u8(self.content_exists);

        if self.content_exists == 1 {
            if let Some(loc) = &self.largest {
                loc.encode(buf)?;
            } else {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "missing largest location").into(),
                );
            }
        }

        buf.put_u8(self.forward);

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

        let track_alias = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track alias"))?;

        if buf.len() < 2 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "flags").into());
        }
        let group_order = buf.split_to(1)[0];
        if group_order == 0 || group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }
        let content_exists = buf.split_to(1)[0];
        if content_exists != 0 && content_exists != 1 {
            return Err(
                IoError::new(ErrorKind::InvalidData, "invalid content exists value").into(),
            );
        }

        let largest = if content_exists == 1 {
            Some(Location::decode(buf)?)
        } else {
            None
        };

        if buf.len() < 1 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "forward").into());
        }
        let forward = buf.split_to(1)[0];
        if forward != 0 && forward != 1 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid forward value").into());
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

        Ok(Publish {
            request_id,
            track_namespace,
            track_name,
            track_alias,
            group_order,
            content_exists,
            largest,
            forward,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_with_content() {
        let msg = Publish {
            request_id: 1,
            track_namespace: 2,
            track_name: "video".into(),
            track_alias: 3,
            group_order: 1,
            content_exists: 1,
            largest: Some(Location {
                group: 10,
                object: 5,
            }),
            forward: 1,
            parameters: vec![Parameter {
                parameter_type: 4,
                value: vec![7, 8],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Publish::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_without_content() {
        let msg = Publish {
            request_id: 5,
            track_namespace: 7,
            track_name: "audio".into(),
            track_alias: 8,
            group_order: 1,
            content_exists: 0,
            largest: None,
            forward: 0,
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Publish::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
