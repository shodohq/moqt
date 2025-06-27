use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
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

pub struct Codec;

impl Encoder<ControlMessage> for Codec {
    type Error = Error;

    fn encode(&mut self, item: ControlMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            ControlMessage::ClientSetup(msg) => {
                VarInt.encode(ControlMessageType::ClientSetup as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::ServerSetup(msg) => {
                VarInt.encode(ControlMessageType::ServerSetup as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
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
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::MaxRequestId(msg) => {
                VarInt.encode(ControlMessageType::MaxRequestId as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
            }
            ControlMessage::RequestsBlocked(msg) => {
                VarInt.encode(ControlMessageType::RequestsBlocked as u64, dst)?;
                let mut buf = BytesMut::new();
                msg.encode(&mut buf)?;
                VarInt.encode(buf.len() as u64, dst)?;
                dst.put(buf);
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

impl Decoder for Codec {
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

/// Variable-Length Integer Encoding
///
/// https://datatracker.ietf.org/doc/html/rfc9000#name-variable-length-integer-enc
pub struct VarInt;

impl Encoder<u64> for VarInt {
    type Error = Error;

    fn encode(&mut self, item: u64, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item < (1 << 6) {
            dst.put_u8(item as u8);
        } else if item < (1 << 14) {
            dst.put_u8(0x40 | ((item >> 8) as u8));
            dst.put_u8(item as u8);
        } else if item < (1 << 30) {
            dst.put_u8(0x80 | ((item >> 24) as u8));
            dst.put_u8(((item >> 16) & 0xff) as u8);
            dst.put_u8(((item >> 8) & 0xff) as u8);
            dst.put_u8(item as u8);
        } else if item < (1 << 62) {
            dst.put_u8(0xC0 | ((item >> 56) as u8));
            dst.put_u8(((item >> 48) & 0xff) as u8);
            dst.put_u8(((item >> 40) & 0xff) as u8);
            dst.put_u8(((item >> 32) & 0xff) as u8);
            dst.put_u8(((item >> 24) & 0xff) as u8);
            dst.put_u8(((item >> 16) & 0xff) as u8);
            dst.put_u8(((item >> 8) & 0xff) as u8);
            dst.put_u8(item as u8);
        } else {
            return Err(Error::VarIntRange);
        }
        Ok(())
    }
}

impl Decoder for VarInt {
    type Item = u64;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let first = match src.first() {
            Some(v) => *v,
            None => return Ok(None),
        };

        let prefix = first >> 6;
        let len = 1usize << prefix;

        if src.len() < len {
            return Ok(None);
        }

        let value = match len {
            1 => (first & 0x3f) as u64,
            2 => {
                let b1 = src[1] as u64;
                ((first as u64 & 0x3f) << 8) | b1
            }
            4 => {
                ((first as u64 & 0x3f) << 24)
                    | ((src[1] as u64) << 16)
                    | ((src[2] as u64) << 8)
                    | src[3] as u64
            }
            8 => {
                ((first as u64 & 0x3f) << 56)
                    | ((src[1] as u64) << 48)
                    | ((src[2] as u64) << 40)
                    | ((src[3] as u64) << 32)
                    | ((src[4] as u64) << 24)
                    | ((src[5] as u64) << 16)
                    | ((src[6] as u64) << 8)
                    | src[7] as u64
            }
            _ => unreachable!(),
        };

        let _ = src.split_to(len);
        Ok(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn encode_examples() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (63, &[0x3f]),
            (64, &[0x40, 0x40]),
            (16383, &[0x7f, 0xff]),
            (16384, &[0x80, 0x00, 0x40, 0x00]),
            (1073741823, &[0xbf, 0xff, 0xff, 0xff]),
            (
                1073741824,
                &[0xc0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00],
            ),
        ];

        for (value, expected) in cases {
            let mut buf = BytesMut::new();
            VarInt.encode(*value, &mut buf).unwrap();
            assert_eq!(buf.as_ref(), *expected);
        }
    }

    #[test]
    fn decode_examples() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (63, &[0x3f]),
            (64, &[0x40, 0x40]),
            (16383, &[0x7f, 0xff]),
            (16384, &[0x80, 0x00, 0x40, 0x00]),
            (1073741823, &[0xbf, 0xff, 0xff, 0xff]),
            (
                1073741824,
                &[0xc0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00],
            ),
        ];

        for (expected, bytes) in cases {
            let mut buf = BytesMut::from(*bytes);
            let value = VarInt.decode(&mut buf).unwrap().unwrap();
            assert_eq!(value, *expected);
            assert!(buf.is_empty());
        }
    }

    #[test]
    fn decode_incomplete_returns_none() {
        let mut buf = BytesMut::from(&b"\x40"[..]);
        assert!(VarInt.decode(&mut buf).unwrap().is_none());
        assert_eq!(buf.len(), 1);
    }
}
