use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::{Location, Parameter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscribeUpdate {
    pub request_id: u64,
    pub start_location: Location,
    pub end_group: u64,
    pub subscriber_priority: u8,
    pub forward: u8,
    pub parameters: Vec<Parameter>,
}

impl SubscribeUpdate {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.request_id, buf)?;
        self.start_location.encode(buf)?;
        vi.encode(self.end_group, buf)?;

        buf.put_u8(self.subscriber_priority);
        if self.forward != 0 && self.forward != 1 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid forward value").into());
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

        let mut vi = crate::codec::VarInt;

        let request_id = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "request id"))?;
        let start_location = Location::decode(buf)?;
        let end_group = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "end group"))?;

        if buf.len() < 2 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "flags").into());
        }
        let subscriber_priority = buf.split_to(1)[0];
        let forward_byte = buf.split_to(1)[0];
        if forward_byte != 0 && forward_byte != 1 {
            return Err(IoError::new(ErrorKind::InvalidData, "invalid forward value").into());
        }
        let forward = forward_byte;

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

        Ok(SubscribeUpdate {
            request_id,
            start_location,
            end_group,
            subscriber_priority,
            forward,
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
        let msg = SubscribeUpdate {
            request_id: 1,
            start_location: Location { group: 5, object: 2 },
            end_group: 10,
            subscriber_priority: 3,
            forward: 1,
            parameters: vec![Parameter { parameter_type: 1, value: vec![42] }],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = SubscribeUpdate::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_fails_on_invalid_forward() {
        let mut buf = BytesMut::new();
        let mut vi = crate::codec::VarInt;
        vi.encode(1, &mut buf).unwrap(); // request_id
        Location { group: 1, object: 0 }.encode(&mut buf).unwrap();
        vi.encode(0, &mut buf).unwrap(); // end_group
        buf.put_u8(0); // subscriber_priority
        buf.put_u8(2); // invalid forward
        vi.encode(0, &mut buf).unwrap(); // no parameters

        assert!(SubscribeUpdate::decode(&mut buf).is_err());
    }
}
