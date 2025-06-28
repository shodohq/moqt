use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::codec::{Decode, Encode, VarInt};

pub struct WithLengthCodec<T> {
    _marker: std::marker::PhantomData<T>,
}

impl WithLengthCodec<()> {
    pub fn new() -> Self {
        WithLengthCodec {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Encode, U> Encoder<T> for WithLengthCodec<U> {
    type Error = crate::error::Error;

    fn encode(&mut self, item: T, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let mut buf = BytesMut::new();
        item.encode(&mut buf)?;

        let len = buf.len() as u64;
        VarInt.encode(len, dst)?;
        dst.extend(buf);

        Ok(())
    }
}

impl<T: Decode> Decoder for WithLengthCodec<T> {
    type Item = T;
    type Error = crate::error::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 1 {
            return Ok(None);
        }

        if let Some(len) = VarInt.decode(src)? {
            let len = len as usize;
            if src.len() < len {
                // TODO: handle this case properly
                todo!()
            }
            let item = T::decode(&mut src.split_to(len))?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}
