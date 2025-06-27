use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    pub parameter_type: u64,
    pub value: Vec<u8>,
}

impl Parameter {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.parameter_type, buf)?;

        if self.parameter_type % 2 == 0 {
            // even types contain a varint value directly
            if self.value.is_empty() || self.value.len() > 8 {
                return Err(crate::error::Error::ProtocolViolation {
                    reason: "invalid varint parameter value".into(),
                });
            }
            buf.put_slice(&self.value);
        } else {
            if self.value.len() > 0xFFFF {
                return Err(crate::error::Error::ProtocolViolation {
                    reason: "parameter value length exceeded".into(),
                });
            }
            vi.encode(self.value.len() as u64, buf)?;
            buf.put_slice(&self.value);
        }

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        let parameter_type = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameter type"))?;

        let value = if parameter_type % 2 == 0 {
            let val = vi
                .decode(buf)?
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameter value"))?;
            let mut tmp = BytesMut::new();
            vi.encode(val, &mut tmp)?;
            tmp.to_vec()
        } else {
            let len = vi
                .decode(buf)?
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameter len"))?
                as usize;
            if len > 0xFFFF {
                return Err(crate::error::Error::ProtocolViolation {
                    reason: "parameter value length exceeded".into(),
                });
            }
            if buf.len() < len {
                return Err(IoError::new(ErrorKind::UnexpectedEof, "parameter value").into());
            }
            buf.split_to(len).to_vec()
        };

        Ok(Parameter {
            parameter_type,
            value,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub group: u64,
    pub object: u64,
}

impl Location {
    pub fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;
        vi.encode(self.group, buf)?;
        vi.encode(self.object, buf)?;
        Ok(())
    }

    pub fn decode(buf: &mut bytes::BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;
        let group = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "location group"))?;
        let object = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "location object"))?;
        Ok(Location { group, object })
    }
}
