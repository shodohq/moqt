use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscribeOk {
    pub request_id: u64,
    pub track_alias: u64,
    pub expires: u64,
    pub group_order: u8,
    pub content_exists: bool,
    pub largest_location: Option<Location>,
    pub parameters: Vec<Parameter>,
}

impl SubscribeOk {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.request_id, buf)?;
        vi.encode(self.track_alias, buf)?;
        vi.encode(self.expires, buf)?;

        if self.group_order == 0 || self.group_order > 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid group order",
            )
            .into());
        }
        buf.put_u8(self.group_order);
        buf.put_u8(if self.content_exists { 1 } else { 0 });

        if self.content_exists {
            if let Some(loc) = &self.largest_location {
                loc.encode(buf)?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "missing largest location",
                )
                .into());
            }
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

        let mut vi = crate::codec::VarInt;

        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;
        let track_alias = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track alias"))?;
        let expires = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "expires"))?;

        if buf.len() < 2 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "flags").into());
        }
        let group_order = buf.split_to(1)[0];
        if group_order == 0 || group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }
        let content_exists_byte = buf.split_to(1)[0];
        let content_exists = match content_exists_byte {
            0 => false,
            1 => true,
            _ => {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "invalid content exists value").into(),
                );
            }
        };

        let largest_location = if content_exists {
            Some(Location::decode(buf)?)
        } else {
            None
        };

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
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameter length"))?
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

        Ok(SubscribeOk {
            request_id,
            track_alias,
            expires,
            group_order,
            content_exists,
            largest_location,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_with_location() {
        let msg = SubscribeOk {
            request_id: 1,
            track_alias: 2,
            expires: 500,
            group_order: 1,
            content_exists: true,
            largest_location: Some(Location {
                group: 10,
                object: 7,
            }),
            parameters: vec![Parameter {
                parameter_type: 1,
                value: vec![42],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeOk::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_without_location() {
        let msg = SubscribeOk {
            request_id: 3,
            track_alias: 4,
            expires: 0,
            group_order: 2,
            content_exists: false,
            largest_location: None,
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeOk::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
