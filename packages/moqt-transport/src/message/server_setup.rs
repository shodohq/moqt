use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::SetupParameter;

/// Representation of a SERVER_SETUP message body.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerSetup {
    /// Selected protocol version.
    pub version: u32,
    /// Raw setup parameters.
    pub parameters: Vec<SetupParameter>,
}

impl ServerSetup {
    /// Encode the SERVER_SETUP message body into the provided buffer.
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        vi.encode(self.version as u64, buf)?;
        vi.encode(self.parameters.len() as u64, buf)?;
        for p in &self.parameters {
            vi.encode(p.parameter_type, buf)?;
            vi.encode(p.value.len() as u64, buf)?;
            buf.put_slice(&p.value);
        }

        Ok(())
    }

    /// Decode a SERVER_SETUP message body from the provided buffer.
    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        let version = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "version"))?
            as u32;

        let params_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameters"))?
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
            parameters.push(SetupParameter { parameter_type: ty, value });
        }

        Ok(ServerSetup { version, parameters })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = ServerSetup {
            version: 1,
            parameters: vec![
                SetupParameter { parameter_type: 0x02, value: vec![5] },
                SetupParameter { parameter_type: 0x04, value: vec![1, 2] },
            ],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = ServerSetup::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
