use bytes::BytesMut;
pub struct SubscribeUpdate {}

impl SubscribeUpdate {
    pub fn encode(&self, _buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        todo!()
    }

    pub fn decode(_buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        todo!()
    }
}
