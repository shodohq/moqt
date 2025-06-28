use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    codec::{Decode, VarInt, WithLengthCodec},
    error::Error,
    message::{
        Announce, AnnounceCancel, AnnounceError, AnnounceOk, ClientSetup, ControlMessage,
        ControlMessageType, Fetch, FetchCancel, FetchError, FetchOk, Goaway, MaxRequestId, Publish,
        PublishError, PublishOk, RequestsBlocked, ServerSetup, Subscribe, SubscribeAnnounces,
        SubscribeAnnouncesError, SubscribeAnnouncesOk, SubscribeDone, SubscribeError, SubscribeOk,
        SubscribeUpdate, TrackStatus, TrackStatusRequest, Unannounce, Unsubscribe,
        UnsubscribeAnnounces,
    },
};

pub struct ControlMessageCodec;

impl Encoder<ControlMessage> for ControlMessageCodec {
    type Error = Error;

    fn encode(&mut self, item: ControlMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut with_length = WithLengthCodec::new();

        match item {
            ControlMessage::ClientSetup(msg) => {
                VarInt.encode(ControlMessageType::ClientSetup as u64, dst)?;
                with_length.encode(msg, dst)?;
            }
            ControlMessage::ServerSetup(msg) => {
                VarInt.encode(ControlMessageType::ServerSetup as u64, dst)?;
                with_length.encode(msg, dst)?;
            }
            ControlMessage::Subscribe(msg) => {
                VarInt.encode(ControlMessageType::Subscribe as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeAnnounces(msg) => {
                VarInt.encode(ControlMessageType::SubscribeAnnounces as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeAnnouncesOk(msg) => {
                VarInt.encode(ControlMessageType::SubscribeAnnouncesOk as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeAnnouncesError(msg) => {
                VarInt.encode(ControlMessageType::SubscribeAnnouncesError as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeOk(msg) => {
                VarInt.encode(ControlMessageType::SubscribeOk as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeError(msg) => {
                VarInt.encode(ControlMessageType::SubscribeError as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeUpdate(msg) => {
                VarInt.encode(ControlMessageType::SubscribeUpdate as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::Unsubscribe(msg) => {
                VarInt.encode(ControlMessageType::Unsubscribe as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::UnsubscribeAnnounces(msg) => {
                VarInt.encode(ControlMessageType::UnsubscribeAnnounces as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::SubscribeDone(msg) => {
                VarInt.encode(ControlMessageType::SubscribeDone as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::Publish(msg) => {
                VarInt.encode(ControlMessageType::Publish as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::PublishOk(msg) => {
                VarInt.encode(ControlMessageType::PublishOk as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::PublishError(msg) => {
                VarInt.encode(ControlMessageType::PublishError as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::Fetch(msg) => {
                VarInt.encode(ControlMessageType::Fetch as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::FetchOk(msg) => {
                VarInt.encode(ControlMessageType::FetchOk as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::FetchError(msg) => {
                VarInt.encode(ControlMessageType::FetchError as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::FetchCancel(msg) => {
                VarInt.encode(ControlMessageType::FetchCancel as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::Goaway(msg) => {
                VarInt.encode(ControlMessageType::Goaway as u64, dst)?;
                with_length.encode(msg, dst)?;
            }
            ControlMessage::MaxRequestId(msg) => {
                VarInt.encode(ControlMessageType::MaxRequestId as u64, dst)?;
                with_length.encode(msg, dst)?;
            }
            ControlMessage::RequestsBlocked(msg) => {
                VarInt.encode(ControlMessageType::RequestsBlocked as u64, dst)?;
                with_length.encode(msg, dst)?;
            }
            ControlMessage::TrackStatus(msg) => {
                VarInt.encode(ControlMessageType::TrackStatus as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::TrackStatusRequest(msg) => {
                VarInt.encode(ControlMessageType::TrackStatusRequest as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::Announce(msg) => {
                VarInt.encode(ControlMessageType::Announce as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::AnnounceOk(msg) => {
                VarInt.encode(ControlMessageType::AnnounceOk as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::AnnounceError(msg) => {
                VarInt.encode(ControlMessageType::AnnounceError as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::Unannounce(msg) => {
                VarInt.encode(ControlMessageType::Unannounce as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::AnnounceCancel(msg) => {
                VarInt.encode(ControlMessageType::AnnounceCancel as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
        }
        Ok(())
    }
}

impl Decoder for ControlMessageCodec {
    type Item = ControlMessage;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let msg_type = match VarInt.decode(src)? {
            Some(v) => v,
            None => return Ok(None),
        };
        let len = match VarInt.decode(src)? {
            Some(v) => v as usize,
            None => return Ok(None),
        };
        if src.len() < len {
            return Ok(None);
        }
        let mut payload = src.split_to(len);
        let message = match ControlMessageType::try_from(msg_type)? {
            ControlMessageType::ClientSetup => {
                ControlMessage::ClientSetup(ClientSetup::decode(&mut payload)?)
            }
            ControlMessageType::ServerSetup => {
                ControlMessage::ServerSetup(ServerSetup::decode(&mut payload)?)
            }
            ControlMessageType::Subscribe => {
                ControlMessage::Subscribe(Subscribe::decode(&mut payload)?)
            }
            ControlMessageType::SubscribeAnnounces => {
                ControlMessage::SubscribeAnnounces(SubscribeAnnounces::decode(&mut payload)?)
            }
            ControlMessageType::SubscribeAnnouncesOk => {
                ControlMessage::SubscribeAnnouncesOk(SubscribeAnnouncesOk::decode(&mut payload)?)
            }
            ControlMessageType::SubscribeAnnouncesError => ControlMessage::SubscribeAnnouncesError(
                SubscribeAnnouncesError::decode(&mut payload)?,
            ),
            ControlMessageType::SubscribeOk => {
                ControlMessage::SubscribeOk(SubscribeOk::decode(&mut payload)?)
            }
            ControlMessageType::SubscribeError => {
                ControlMessage::SubscribeError(SubscribeError::decode(&mut payload)?)
            }
            ControlMessageType::SubscribeUpdate => {
                ControlMessage::SubscribeUpdate(SubscribeUpdate::decode(&mut payload)?)
            }
            ControlMessageType::Unsubscribe => {
                ControlMessage::Unsubscribe(Unsubscribe::decode(&mut payload)?)
            }
            ControlMessageType::UnsubscribeAnnounces => {
                ControlMessage::UnsubscribeAnnounces(UnsubscribeAnnounces::decode(&mut payload)?)
            }
            ControlMessageType::SubscribeDone => {
                ControlMessage::SubscribeDone(SubscribeDone::decode(&mut payload)?)
            }
            ControlMessageType::Publish => ControlMessage::Publish(Publish::decode(&mut payload)?),
            ControlMessageType::PublishOk => {
                ControlMessage::PublishOk(PublishOk::decode(&mut payload)?)
            }
            ControlMessageType::PublishError => {
                ControlMessage::PublishError(PublishError::decode(&mut payload)?)
            }
            ControlMessageType::Fetch => ControlMessage::Fetch(Fetch::decode(&mut payload)?),
            ControlMessageType::FetchOk => ControlMessage::FetchOk(FetchOk::decode(&mut payload)?),
            ControlMessageType::FetchError => {
                ControlMessage::FetchError(FetchError::decode(&mut payload)?)
            }
            ControlMessageType::FetchCancel => {
                ControlMessage::FetchCancel(FetchCancel::decode(&mut payload)?)
            }
            ControlMessageType::Goaway => ControlMessage::Goaway(Goaway::decode(&mut payload)?),
            ControlMessageType::MaxRequestId => {
                ControlMessage::MaxRequestId(MaxRequestId::decode(&mut payload)?)
            }
            ControlMessageType::RequestsBlocked => {
                ControlMessage::RequestsBlocked(RequestsBlocked::decode(&mut payload)?)
            }
            ControlMessageType::TrackStatus => {
                ControlMessage::TrackStatus(TrackStatus::decode(&mut payload)?)
            }
            ControlMessageType::TrackStatusRequest => {
                ControlMessage::TrackStatusRequest(TrackStatusRequest::decode(&mut payload)?)
            }
            ControlMessageType::Announce => {
                ControlMessage::Announce(Announce::decode(&mut payload)?)
            }
            ControlMessageType::AnnounceOk => {
                ControlMessage::AnnounceOk(AnnounceOk::decode(&mut payload)?)
            }
            ControlMessageType::AnnounceError => {
                ControlMessage::AnnounceError(AnnounceError::decode(&mut payload)?)
            }
            ControlMessageType::Unannounce => {
                ControlMessage::Unannounce(Unannounce::decode(&mut payload)?)
            }
            ControlMessageType::AnnounceCancel => {
                ControlMessage::AnnounceCancel(AnnounceCancel::decode(&mut payload)?)
            }
        };
        if !payload.is_empty() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "excess payload").into(),
            );
        }
        Ok(Some(message))
    }
}

#[cfg(test)]
mod tests {
    use super::ControlMessageCodec;
    use crate::message::{ControlMessage, MaxRequestId, RequestsBlocked};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn codec_requests_blocked_roundtrip() {
        let mut codec = ControlMessageCodec;
        let msg = ControlMessage::RequestsBlocked(RequestsBlocked {
            maximum_request_id: 42,
        });

        let mut buf = BytesMut::new();
        codec.encode(msg, &mut buf).unwrap();
        assert_eq!(buf.as_ref(), &[0x1A, 0x01, 0x2A]);

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            ControlMessage::RequestsBlocked(rb) => {
                assert_eq!(rb.maximum_request_id, 42);
            }
            _ => panic!("unexpected message"),
        }
        assert!(buf.is_empty());
    }

    #[test]
    fn codec_max_request_id_roundtrip() {
        let mut codec = ControlMessageCodec;
        let msg = ControlMessage::MaxRequestId(MaxRequestId { request_id: 5 });

        let mut buf = BytesMut::new();
        codec.encode(msg, &mut buf).unwrap();
        assert_eq!(buf.as_ref(), &[0x15, 0x01, 0x05]);

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            ControlMessage::MaxRequestId(mr) => {
                assert_eq!(mr.request_id, 5);
            }
            _ => panic!("unexpected message"),
        }
        assert!(buf.is_empty());
    }
}
