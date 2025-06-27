use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::Parameter;

/// CLIENT_SETUP
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
/// ```text
/// CLIENT_SETUP Message {
///   Type (i) = 0x20,
///   Length (16),
///   Number of Supported Versions (i),
///   Supported Versions (i) ...,
///   Number of Parameters (i),
///   Setup Parameters (..) ...,
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientSetup {
    pub supported_versions: Vec<u32>,
    pub setup_parameters: Vec<Parameter>,
}

impl ClientSetup {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::coding::VarInt;

        // Supported Versions
        vi.encode(self.supported_versions.len() as u64, buf)?;
        for v in &self.supported_versions {
            vi.encode(*v as u64, buf)?;
        }

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

        let mut vi = crate::coding::VarInt;

        // Supported Versions
        let versions_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "versions"))?
            as usize;
        let mut versions = Vec::with_capacity(versions_len);
        for _ in 0..versions_len {
            let v = vi
                .decode(buf)?
                .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "version"))?;
            versions.push(v as u32);
        }

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
            parameters.push(Parameter {
                parameter_type: ty,
                value,
            });
        }

        Ok(ClientSetup {
            supported_versions: versions,
            setup_parameters: parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Parameter;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = ClientSetup {
            supported_versions: vec![1, 0xff00000d],
            setup_parameters: vec![
                Parameter {
                    parameter_type: 0x01,
                    value: b"/".to_vec(),
                },
                Parameter {
                    parameter_type: 0x02,
                    value: vec![5],
                },
            ],
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = ClientSetup::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }
}
