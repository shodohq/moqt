mod announce;
mod announce_cancel;
mod announce_error;
mod announce_ok;
mod client_setup;
mod fetch;
mod fetch_cancel;
mod fetch_error;
mod fetch_ok;
mod goaway;
mod max_request_id;
mod publish;
mod publish_error;
mod publish_ok;
mod requests_blocked;
mod server_setup;
mod subscribe;
mod subscribe_announces;
mod subscribe_announces_error;
mod subscribe_announces_ok;
mod subscribe_done;
mod subscribe_error;
mod subscribe_ok;
mod subscribe_update;
mod track_status;
mod track_status_request;
mod unannounce;
mod unsubscribe;
mod unsubscribe_announces;

pub use announce::*;
pub use announce_cancel::*;
pub use announce_error::*;
pub use announce_ok::*;
pub use client_setup::*;
pub use fetch::*;
pub use fetch_cancel::*;
pub use fetch_error::*;
pub use fetch_ok::*;
pub use goaway::*;
pub use max_request_id::*;
pub use publish::*;
pub use publish_error::*;
pub use publish_ok::*;
pub use requests_blocked::*;
pub use server_setup::*;
pub use subscribe::*;
pub use subscribe_announces::*;
pub use subscribe_announces_error::*;
pub use subscribe_announces_ok::*;
pub use subscribe_done::*;
pub use subscribe_error::*;
pub use subscribe_ok::*;
pub use subscribe_update::*;
pub use track_status::*;
pub use track_status_request::*;
pub use unannounce::*;
pub use unsubscribe::*;
pub use unsubscribe_announces::*;

/// Control Messages
/// https://datatracker.ietf.org/doc/html/draft-ietf-moq-transport-12#name-control-messages
///
/// MOQT uses a single bidirectional stream to exchange control messages,
/// as defined in Section 3.3. Every single message on the control
/// stream is formatted as follows:
///
/// MOQT Control Message {
///   Message Type (i),
///   Message Length (16),
///   Message Payload (..),
/// }
pub enum ControlMessage {
    ClientSetup(ClientSetup),
    ServerSetup(ServerSetup),
    Goaway(Goaway),
    MaxRequestId(MaxRequestId),
    RequestsBlocked(RequestsBlocked),
    Subscribe(Subscribe),
    SubscribeOk(SubscribeOk),
    SubscribeError(SubscribeError),
    SubscribeUpdate(SubscribeUpdate),
    Unsubscribe(Unsubscribe),
    SubscribeDone(SubscribeDone),
    Publish(Publish),
    PublishOk(PublishOk),
    PublishError(PublishError),
    Fetch(Fetch),
    FetchOk(FetchOk),
    FetchError(FetchError),
    FetchCancel(FetchCancel),
    TrackStatusRequest(TrackStatusRequest),
    TrackStatus(TrackStatus),
    Announce(Announce),
    AnnounceOk(AnnounceOk),
    AnnounceError(AnnounceError),
    Unannounce(Unannounce),
    AnnounceCancel(AnnounceCancel),
    SubscribeAnnounces(SubscribeAnnounces),
    SubscribeAnnouncesOk(SubscribeAnnouncesOk),
    SubscribeAnnouncesError(SubscribeAnnouncesError),
    UnsubscribeAnnounces(UnsubscribeAnnounces),
}

/// https://datatracker.ietf.org/doc/html/draft-ietf-moq-transport-12#table-2
pub enum ControlMessageType {
    ClientSetup = 0x20,
    ServerSetup = 0x21,
    Goaway = 0x10,
    MaxRequestId = 0x15,
    RequestsBlocked = 0x1A,
    Subscribe = 0x03,
    SubscribeOk = 0x04,
    SubscribeError = 0x05,
    SubscribeUpdate = 0x02,
    Unsubscribe = 0x0A,
    SubscribeDone = 0x0B,
    Publish = 0x1D,
    PublishOk = 0x1E,
    PublishError = 0x1F,
    Fetch = 0x16,
    FetchOk = 0x18,
    FetchError = 0x19,
    FetchCancel = 0x17,
    TrackStatusRequest = 0x0D,
    TrackStatus = 0x0E,
    Announce = 0x06,
    AnnounceOk = 0x07,
    AnnounceError = 0x08,
    Unannounce = 0x09,
    AnnounceCancel = 0x0C,
    SubscribeAnnounces = 0x11,
    SubscribeAnnouncesOk = 0x12,
    SubscribeAnnouncesError = 0x13,
    UnsubscribeAnnounces = 0x14,
}

impl TryFrom<u64> for ControlMessageType {
    type Error = crate::error::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(ControlMessageType::ClientSetup),
            0x21 => Ok(ControlMessageType::ServerSetup),
            0x10 => Ok(ControlMessageType::Goaway),
            0x15 => Ok(ControlMessageType::MaxRequestId),
            0x1A => Ok(ControlMessageType::RequestsBlocked),
            0x03 => Ok(ControlMessageType::Subscribe),
            0x04 => Ok(ControlMessageType::SubscribeOk),
            0x05 => Ok(ControlMessageType::SubscribeError),
            0x02 => Ok(ControlMessageType::SubscribeUpdate),
            0x0A => Ok(ControlMessageType::Unsubscribe),
            0x0B => Ok(ControlMessageType::SubscribeDone),
            0x1D => Ok(ControlMessageType::Publish),
            0x1E => Ok(ControlMessageType::PublishOk),
            0x1F => Ok(ControlMessageType::PublishError),
            0x16 => Ok(ControlMessageType::Fetch),
            0x18 => Ok(ControlMessageType::FetchOk),
            0x19 => Ok(ControlMessageType::FetchError),
            0x17 => Ok(ControlMessageType::FetchCancel),
            0x0D => Ok(ControlMessageType::TrackStatusRequest),
            0x0E => Ok(ControlMessageType::TrackStatus),
            0x06 => Ok(ControlMessageType::Announce),
            0x07 => Ok(ControlMessageType::AnnounceOk),
            0x08 => Ok(ControlMessageType::AnnounceError),
            0x09 => Ok(ControlMessageType::Unannounce),
            0x0C => Ok(ControlMessageType::AnnounceCancel),
            0x11 => Ok(ControlMessageType::SubscribeAnnounces),
            0x12 => Ok(ControlMessageType::SubscribeAnnouncesOk),
            0x13 => Ok(ControlMessageType::SubscribeAnnouncesError),
            0x14 => Ok(ControlMessageType::UnsubscribeAnnounces),
            _ => Err(crate::error::Error::UnknownMessageType),
        }
    }
}
