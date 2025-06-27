use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
    pub parameter_type: u64,
    pub value: Vec<u8>,
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
