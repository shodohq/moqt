use bytes::BytesMut;
pub struct TrackStatus {}

impl TrackStatus {
    pub fn encode(&self, _buf: &mut BytesMut) -> Result<(), crate::error::Error> {
        todo!()
    }

    pub fn decode(_buf: &mut BytesMut) -> Result<Self, crate::error::Error> {
        todo!()
    }
}
