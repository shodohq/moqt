use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TrackStatus {
    pub request_id: u64,
    pub status_code: u64,
    pub largest_location: Location,
    pub parameters: Vec<Parameter>,
}

impl TrackStatus {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::coding::VarInt;

        if !matches!(self.status_code, 0x00 | 0x01 | 0x02 | 0x03 | 0x04) {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid status code").into());
        }

        if matches!(self.status_code, 0x01 | 0x02) {
            if self.largest_location.group != 0 || self.largest_location.object != 0 {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "largest location must be zero").into(),
                );
            }
            if !self.parameters.is_empty() {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "parameters must be empty").into(),
                );
            }
        }

        vi.encode(self.request_id, buf)?;
        vi.encode(self.status_code, buf)?;
        self.largest_location.encode(buf)?;

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
        let status_code = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "status code"))?;

        if !matches!(status_code, 0x00 | 0x01 | 0x02 | 0x03 | 0x04) {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid status code").into());
        }

        let largest_location = Location::decode(buf)?;

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

        if matches!(status_code, 0x01 | 0x02) {
            if largest_location.group != 0 || largest_location.object != 0 {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "largest location must be zero").into(),
                );
            }
            if !parameters.is_empty() {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "parameters must be empty").into(),
                );
            }
        }

        Ok(TrackStatus {
            request_id,
            status_code,
            largest_location,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_progress() {
        let msg = TrackStatus {
            request_id: 1,
            status_code: 0x00,
            largest_location: Location {
                group: 10,
                object: 5,
            },
            parameters: vec![Parameter {
                parameter_type: 2,
                value: vec![42],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = TrackStatus::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_decode_roundtrip_not_started() {
        let msg = TrackStatus {
            request_id: 5,
            status_code: 0x02,
            largest_location: Location {
                group: 0,
                object: 0,
            },
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = TrackStatus::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn encode_fails_on_nonzero_location_for_nonexistent() {
        let msg = TrackStatus {
            request_id: 1,
            status_code: 0x01,
            largest_location: Location {
                group: 1,
                object: 0,
            },
            parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        assert!(msg.encode(&mut buf).is_err());
    }

    #[test]
    fn decode_fails_on_invalid_status_code() {
        let mut buf = BytesMut::new();
        let mut vi = crate::coding::VarInt;
        vi.encode(1, &mut buf).unwrap(); // request_id
        vi.encode(0x09, &mut buf).unwrap(); // invalid status code
        Location {
            group: 0,
            object: 0,
        }
        .encode(&mut buf)
        .unwrap();
        vi.encode(0, &mut buf).unwrap();

        assert!(TrackStatus::decode(&mut buf).is_err());
    }

    #[test]
    fn decode_fails_on_nonzero_fields_for_not_started() {
        let mut buf = BytesMut::new();
        let mut vi = crate::coding::VarInt;
        vi.encode(1, &mut buf).unwrap(); // request_id
        vi.encode(0x02, &mut buf).unwrap(); // status code (not yet begun)
        Location {
            group: 1,
            object: 0,
        }
        .encode(&mut buf)
        .unwrap();
        vi.encode(1, &mut buf).unwrap(); // parameters len
        vi.encode(1, &mut buf).unwrap(); // param type
        vi.encode(1, &mut buf).unwrap(); // param len
        buf.put_u8(0);

        assert!(TrackStatus::decode(&mut buf).is_err());
    }
}
