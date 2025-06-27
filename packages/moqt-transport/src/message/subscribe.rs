use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Subscribe {
    pub request_id: u64,
    pub track_namespace: u64,
    pub track_name: String,
    pub subscriber_priority: u8,
    pub group_order: u8,
    pub forward: u8,
    pub filter_type: u64,
    pub start_location: Option<Location>,
    pub end_group: Option<u64>,
    pub parameters: Vec<Parameter>,
}

impl Subscribe {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.request_id, buf)?;
        vi.encode(self.track_namespace, buf)?;

        vi.encode(self.track_name.len() as u64, buf)?;
        buf.put_slice(self.track_name.as_bytes());

        buf.put_u8(self.subscriber_priority);

        if self.group_order > 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid group order",
            )
            .into());
        }
        buf.put_u8(self.group_order);

        if self.forward != 0 && self.forward != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid forward value",
            )
            .into());
        }
        buf.put_u8(self.forward);

        if !matches!(self.filter_type, 0x1 | 0x2 | 0x3 | 0x4) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid filter type",
            )
            .into());
        }
        vi.encode(self.filter_type, buf)?;

        if matches!(self.filter_type, 0x3 | 0x4) {
            if let Some(loc) = &self.start_location {
                loc.encode(buf)?;
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "missing start location").into());
            }
        }

        if self.filter_type == 0x4 {
            if let Some(end) = self.end_group {
                vi.encode(end, buf)?;
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "missing end group").into());
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

        if buf.len() < 3 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "flags").into());
        }
        let subscriber_priority = buf.split_to(1)[0];
        let group_order = buf.split_to(1)[0];
        if group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }
        let forward = buf.split_to(1)[0];
        if forward != 0 && forward != 1 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid forward value").into());
        }

        let filter_type = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "filter type"))?;
        if !matches!(filter_type, 0x1 | 0x2 | 0x3 | 0x4) {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid filter type").into());
        }

        let start_location = if matches!(filter_type, 0x3 | 0x4) {
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
            parameters.push(Parameter { parameter_type: ty, value });
        }

        Ok(Subscribe {
            request_id,
            track_namespace,
            track_name,
            subscriber_priority,
            group_order,
            forward,
            filter_type,
            start_location,
            end_group,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_absolute_range() {
        let msg = Subscribe {
            request_id: 1,
            track_namespace: 2,
            track_name: "video".into(),
            subscriber_priority: 3,
            group_order: 1,
            forward: 1,
            filter_type: 0x4,
            start_location: Some(Location { group: 10, object: 5 }),
            end_group: Some(20),
            parameters: vec![Parameter { parameter_type: 1, value: vec![42] }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Subscribe::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_largest_object() {
        let msg = Subscribe {
            request_id: 5,
            track_namespace: 7,
            track_name: "audio".into(),
            subscriber_priority: 0,
            group_order: 0,
            forward: 1,
            filter_type: 0x2,
            start_location: None,
            end_group: None,
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Subscribe::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
