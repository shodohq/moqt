#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Transport layer error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to decode message: {0}")]
    Codec(String),

    #[error("Protocol violation: {reason}")]
    ProtocolViolation { reason: String },

    #[error("Subscription failed: {reason}")]
    SubscriptionFailed { code: u64, reason: String },

    #[error("Session closed")]
    SessionClosed,

    #[error("Invalid track alias: {0}")]
    DuplicateTrackAlias(u64),

    #[error("varint out of range")]
    VarIntRange,

    #[error("unknown message type")]
    UnknownMessageType,

    #[error("std::io::Error")]
    Io(#[from] std::io::Error),
}
