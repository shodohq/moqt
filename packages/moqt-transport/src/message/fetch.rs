use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fetch {
    pub request_id: u64,
    pub subscriber_priority: u8,
    pub group_order: u8,
    pub fetch_type: u64,
    pub track_namespace: Option<u64>,
    pub track_name: Option<String>,
    pub start_location: Option<Location>,
    pub end_location: Option<Location>,
    pub joining_request_id: Option<u64>,
    pub joining_start: Option<u64>,
    pub parameters: Vec<Parameter>,
}

impl Fetch {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        vi.encode(self.request_id, buf)?;
        buf.put_u8(self.subscriber_priority);
        if self.group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }
        buf.put_u8(self.group_order);
        vi.encode(self.fetch_type, buf)?;

        match self.fetch_type {
            0x1 => {
                let ns = self.track_namespace.ok_or_else(|| {
                    IoError::new(ErrorKind::InvalidData, "missing track namespace")
                })?;
                let name = self
                    .track_name
                    .as_ref()
                    .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "missing track name"))?;
                let start = self.start_location.as_ref().ok_or_else(|| {
                    IoError::new(ErrorKind::InvalidData, "missing start location")
                })?;
                let end = self
                    .end_location
                    .as_ref()
                    .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "missing end location"))?;

                vi.encode(ns, buf)?;
                vi.encode(name.len() as u64, buf)?;
                buf.put_slice(name.as_bytes());
                start.encode(buf)?;
                end.encode(buf)?;
            }
            0x2 | 0x3 => {
                let join_req = self.joining_request_id.ok_or_else(|| {
                    IoError::new(ErrorKind::InvalidData, "missing joining request id")
                })?;
                let join_start = self
                    .joining_start
                    .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "missing joining start"))?;
                vi.encode(join_req, buf)?;
                vi.encode(join_start, buf)?;
            }
            _ => {
                return Err(IoError::new(ErrorKind::InvalidData, "invalid fetch type").into());
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

        if buf.len() < 2 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "flags").into());
        }
        let subscriber_priority = buf.split_to(1)[0];
        let group_order = buf.split_to(1)[0];
        if group_order > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }

        let fetch_type = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "fetch type"))?;

        let mut track_namespace = None;
        let mut track_name = None;
        let mut start_location = None;
        let mut end_location = None;
        let mut joining_request_id = None;
        let mut joining_start = None;

        match fetch_type {
            0x1 => {
                track_namespace =
                    Some(vi.decode(buf)?.ok_or_else(|| {
                        IoError::new(ErrorKind::UnexpectedEof, "track namespace")
                    })?);
                let name_len = vi
                    .decode(buf)?
                    .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "track name len"))?
                    as usize;
                if buf.len() < name_len {
                    return Err(IoError::new(ErrorKind::UnexpectedEof, "track name").into());
                }
                let name_bytes = buf.split_to(name_len);
                track_name = Some(
                    String::from_utf8(name_bytes.to_vec())
                        .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?,
                );
                start_location = Some(Location::decode(buf)?);
                end_location = Some(Location::decode(buf)?);
            }
            0x2 | 0x3 => {
                joining_request_id =
                    Some(vi.decode(buf)?.ok_or_else(|| {
                        IoError::new(ErrorKind::UnexpectedEof, "joining request id")
                    })?);
                joining_start = Some(
                    vi.decode(buf)?
                        .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "joining start"))?,
                );
            }
            _ => {
                return Err(IoError::new(ErrorKind::InvalidData, "invalid fetch type").into());
            }
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

        Ok(Fetch {
            request_id,
            subscriber_priority,
            group_order,
            fetch_type,
            track_namespace,
            track_name,
            start_location,
            end_location,
            joining_request_id,
            joining_start,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_standalone() {
        let msg = Fetch {
            request_id: 1,
            subscriber_priority: 2,
            group_order: 1,
            fetch_type: 0x1,
            track_namespace: Some(3),
            track_name: Some("video".into()),
            start_location: Some(Location {
                group: 10,
                object: 5,
            }),
            end_location: Some(Location {
                group: 20,
                object: 0,
            }),
            joining_request_id: None,
            joining_start: None,
            parameters: vec![Parameter {
                parameter_type: 4,
                value: vec![7, 8],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Fetch::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_joining() {
        let msg = Fetch {
            request_id: 5,
            subscriber_priority: 0,
            group_order: 0,
            fetch_type: 0x2,
            track_namespace: None,
            track_name: None,
            start_location: None,
            end_location: None,
            joining_request_id: Some(42),
            joining_start: Some(3),
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = Fetch::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
