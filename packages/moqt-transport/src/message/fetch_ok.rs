use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FetchOk {
    pub request_id: u64,
    pub group_order: u8,
    pub end_of_track: bool,
    pub end_location: Location,
    pub parameters: Vec<Parameter>,
}

impl FetchOk {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.request_id, buf)?;
        buf.put_u8(self.group_order);
        buf.put_u8(if self.end_of_track { 1 } else { 0 });

        self.end_location.encode(buf)?;

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
        let group_order_byte = buf.split_to(1)[0];
        let end_of_track_byte = buf.split_to(1)[0];

        if group_order_byte == 0 || group_order_byte > 2 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid group order").into());
        }

        let end_of_track = match end_of_track_byte {
            0 => false,
            1 => true,
            _ => {
                return Err(
                    IoError::new(ErrorKind::InvalidData, "invalid end of track value").into(),
                );
            }
        };

        let end_location = Location::decode(buf)?;

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

        Ok(FetchOk {
            request_id,
            group_order: group_order_byte,
            end_of_track,
            end_location,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::codec::Encoder;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = FetchOk {
            request_id: 1,
            group_order: 1,
            end_of_track: true,
            end_location: Location {
                group: 10,
                object: 5,
            },
            parameters: vec![Parameter {
                parameter_type: 2,
                value: vec![7, 8],
            }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = FetchOk::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_fails_on_invalid_group_order() {
        let mut buf = BytesMut::new();
        let mut vi = crate::codec::VarInt;
        vi.encode(1, &mut buf).unwrap(); // request_id
        buf.put_u8(3); // invalid group order
        buf.put_u8(0); // end_of_track
        Location {
            group: 0,
            object: 0,
        }
        .encode(&mut buf)
        .unwrap();
        vi.encode(0, &mut buf).unwrap(); // no parameters

        assert!(FetchOk::decode(&mut buf).is_err());
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        let mut vi = crate::codec::VarInt;
        vi.encode(10, &mut buf).unwrap(); // only request_id

        match FetchOk::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
