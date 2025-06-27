use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::model::Parameter;

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
/// ```text
/// SERVER_SETUP Message {
///   Type (i) = 0x21,
///   Length (16),
///   Selected Version (i),
///   Number of Parameters (i),
///   Setup Parameters (..) ...,
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerSetup {
    pub selected_version: u32,
    pub setup_parameters: Vec<Parameter>,
}

impl ServerSetup {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        let mut vi = crate::codec::VarInt;

        // Selected Version
        vi.encode(self.selected_version as u64, buf)?;

        // Setup Parameters
        vi.encode(self.setup_parameters.len() as u64, buf)?;
        for p in &self.setup_parameters {
            p.encode(buf)?;
        }

        Ok(())
    }

    pub fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        use std::io::{Error as IoError, ErrorKind};

        let mut vi = crate::codec::VarInt;

        // Selected Version
        let version = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "version"))?;
        if version > u32::MAX as u64 {
            return Err(crate::error::Error::VarIntRange);
        }
        let version = version as u32;

        // Setup Parameters
        let params_len = vi
            .decode(buf)?
            .ok_or_else(|| IoError::new(ErrorKind::UnexpectedEof, "parameters"))?
            as usize;
        let mut parameters = Vec::with_capacity(params_len);
        for _ in 0..params_len {
            parameters.push(Parameter::decode(buf)?);
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
                Parameter {
                    parameter_type: 0x02,
                    value: vec![5],
                },
                Parameter {
                    parameter_type: 0x04,
                    value: vec![1],
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

    #[test]
    fn encode_decode_roundtrip_no_parameters() {
        let msg = ServerSetup {
            selected_version: 1,
            setup_parameters: Vec::new(),
        };

        let mut buf = BytesMut::new();
        msg.encode(&mut buf).unwrap();

        let mut decode_buf = buf.clone();
        let decoded = ServerSetup::decode(&mut decode_buf).unwrap();
        assert!(decode_buf.is_empty());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn decode_incomplete() {
        let mut buf = BytesMut::new();
        match ServerSetup::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn decode_incomplete_parameter_value() {
        use bytes::BufMut;

        let mut buf = BytesMut::new();
        let mut vi = crate::codec::VarInt;
        vi.encode(1, &mut buf).unwrap(); // selected_version
        vi.encode(1, &mut buf).unwrap(); // number of parameters
        vi.encode(0x02, &mut buf).unwrap(); // parameter type (even)
        buf.put_u8(0x40); // start of two-byte varint but missing second byte

        match ServerSetup::decode(&mut buf) {
            Err(crate::error::Error::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            r => panic!("unexpected result: {:?}", r),
        }
    }

    #[test]
    fn codec_roundtrip() {
        use crate::{codec::ControlMessageCodec, message::ControlMessage};

        let msg = ServerSetup {
            selected_version: 1,
            setup_parameters: vec![Parameter {
                parameter_type: 0x02,
                value: vec![5],
            }],
        };

        let mut codec = ControlMessageCodec;
        let mut buf = BytesMut::new();
        codec
            .encode(ControlMessage::ServerSetup(msg.clone()), &mut buf)
            .unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            ControlMessage::ServerSetup(d) => assert_eq!(d, msg),
            _ => panic!("unexpected message type"),
        }
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_selected_version_overflow() {
        let mut buf = BytesMut::new();
        let mut vi = crate::codec::VarInt;

        // Encode a version that does not fit into u32
        vi.encode((u32::MAX as u64) + 1, &mut buf).unwrap();
        vi.encode(0, &mut buf).unwrap(); // zero parameters

        match ServerSetup::decode(&mut buf) {
            Err(crate::error::Error::VarIntRange) => {}
            r => panic!("unexpected result: {:?}", r),
        }
    }
}
