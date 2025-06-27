use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublishOk {
    pub request_id: u64,
    pub forward: u8,
    pub subscriber_priority: u8,
    pub group_order: u8,
    pub filter_type: u64,
    pub start: Option<Location>,
    pub end_group: Option<u64>,
    pub parameters: Vec<Parameter>,
}

impl PublishOk {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        vi.encode(self.request_id, buf)?;

        if self.forward != 0 && self.forward != 1 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid forward value").into());
        }
        buf.put_u8(self.forward);
        buf.put_u8(self.subscriber_priority);
        if self.group_order == 0 || self.group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }
        buf.put_u8(self.group_order);

        if !matches!(self.filter_type, 0x1 | 0x2 | 0x3 | 0x4) {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid filter type").into());
        }
        vi.encode(self.filter_type, buf)?;

        if matches!(self.filter_type, 0x3 | 0x4) {
            if let Some(loc) = &self.start {
                loc.encode(buf)?;
            } else {
                return Err(IoError::new(ErrorKind::InvalidData, "missing start location").into());
            }
        }

        if self.filter_type == 0x4 {
            if let Some(end) = self.end_group {
                vi.encode(end, buf)?;
            } else {
                return Err(IoError::new(ErrorKind::InvalidData, "missing end group").into());
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

        let mut vi = crate::coding::VarInt;

        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;

        if buf.len() < 3 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "flags").into());
        }
        let forward = buf.split_to(1)[0];
        if forward != 0 && forward != 1 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid forward value").into());
        }
        let subscriber_priority = buf.split_to(1)[0];
        let group_order = buf.split_to(1)[0];
        if group_order == 0 || group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }

        let filter_type = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "filter type"))?;
        if !matches!(filter_type, 0x1 | 0x2 | 0x3 | 0x4) {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid filter type").into());
        }

        let start = if matches!(filter_type, 0x3 | 0x4) {
            Some(Location::decode(buf)?)
        } else {
            None
        };

        let end_group = if filter_type == 0x4 {
            Some(
                vi.decode(buf)?
                    .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "end group"))?,
            )
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

        Ok(PublishOk {
            request_id,
            forward,
            subscriber_priority,
            group_order,
            filter_type,
            start,
            end_group,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_with_range() {
        let msg = PublishOk {
            request_id: 1,
            forward: 1,
            subscriber_priority: 5,
            group_order: 1,
            filter_type: 0x4,
            start: Some(Location {
                group: 10,
                object: 2,
            }),
            end_group: Some(20),
            parameters: vec![Parameter {
                parameter_type: 3,
                value: vec![7, 8],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = PublishOk::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_no_range() {
        let msg = PublishOk {
            request_id: 2,
            forward: 0,
            subscriber_priority: 0,
            group_order: 1,
            filter_type: 0x2,
            start: None,
            end_group: None,
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = PublishOk::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
