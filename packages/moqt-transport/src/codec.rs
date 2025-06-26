use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::{error::Error, message::ControlMessage};

pub struct Codec;

impl Encoder<ControlMessage> for Codec {
    type Error = Error;

    fn encode(&mut self, _item: ControlMessage, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Decoder for Codec {
    type Item = ControlMessage;
    type Error = Error;

    fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

/// Variable-Length Integer Encoding
/// https://datatracker.ietf.org/doc/html/rfc9000#name-variable-length-integer-enc
pub struct VarInt;

impl Encoder<u64> for VarInt {
    type Error = Error;

    fn encode(&mut self, _item: u64, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Decoder for VarInt {
    type Item = u64;
    type Error = Error;

    fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}
