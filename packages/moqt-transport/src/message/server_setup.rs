use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::SetupParameter;

/// SERVER_SETUP
///
/// https://datatracker.ietf.org/doc/html/draft-ietf-moq-transport-12#name-client_setup-and-server_set
///
/// The CLIENT_SETUP and SERVER_SETUP messages are the first messages
/// exchanged by the client and the server; they allow the endpoints to
/// establish the mutually supported version and agree on the initial
/// configuration before any objects are exchanged.  It is a sequence of
/// key-value pairs called Setup parameters; the semantics and format of
/// which can vary based on whether the client or server is sending.  To
/// ensure future extensibility of MOQT, endpoints MUST ignore unknown
/// setup parameters.  TODO: describe GREASE for those.
///
/// SERVER_SETUP Message {
///   Type (i) = 0x21,
///   Length (16),
///   Selected Version (i),
///   Number of Parameters (i),
///   Setup Parameters (..) ...,
/// }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerSetup {
    pub selected_version: u32,
    pub setup_parameters: Vec<SetupParameter>,
}

impl ServerSetup {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        // Selected Version
        vi.encode(self.selected_version as u64, buf)?;

        // Setup Parameters
        vi.encode(self.setup_parameters.len() as u64, buf)?;
        for p in &self.setup_parameters {
            vi.encode(p.parameter_type, buf)?;
            vi.encode(p.value.len() as u64, buf)?;
            buf.put_slice(&p.value);
        }

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        // Selected Version
        let version = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "version"))?
            as u32;

        // Setup Parameters
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
            parameters.push(SetupParameter {
                parameter_type: ty,
                value,
            });
        }

        Ok(ServerSetup {
            selected_version: version,
            setup_parameters: parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = ServerSetup {
            selected_version: 1,
            setup_parameters: vec![
                SetupParameter {
                    parameter_type: 0x02,
                    value: vec![5],
                },
                SetupParameter {
                    parameter_type: 0x04,
                    value: vec![1, 2],
                },
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
