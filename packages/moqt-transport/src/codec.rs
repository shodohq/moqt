mod length;
mod message;
mod varint;

pub use length::*;
pub use message::*;
pub use varint::*;

use bytes::BytesMut;

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::error::Error>;
}

pub trait Decode: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, crate::error::Error>;
}
