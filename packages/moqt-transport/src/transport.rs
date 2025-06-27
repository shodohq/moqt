use tokio::io::{AsyncRead, AsyncWrite};
use bytes::Bytes;
use async_trait::async_trait;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub trait UniStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T> UniStream for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

pub trait BiStream: Send {
    type Reader: AsyncRead + Unpin + Send;
    type Writer: AsyncWrite + Unpin + Send;

    fn split(self) -> (Self::Reader, Self::Writer);
}

#[async_trait]
pub trait Transport: Send + Sync {
    type Uni: UniStream;
    type Bi: BiStream;

    async fn open_uni_stream(&mut self) -> Result<Self::Uni, BoxError>;
    async fn accept_uni_stream(&mut self) -> Result<Self::Uni, BoxError>;

    async fn open_bi_stream(&mut self) -> Result<Self::Bi, BoxError>;
    async fn accept_bi_stream(&mut self) -> Result<Self::Bi, BoxError>;

    async fn send_datagram(&mut self, data: Bytes) -> Result<(), BoxError>;
}
