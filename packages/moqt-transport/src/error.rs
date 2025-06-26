#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("std::io::Error")]
    Io(#[from] std::io::Error),
    #[error("varint out of range")]
    VarIntRange,
}
