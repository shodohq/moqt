use std::future::Future;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub trait UniStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T> UniStream for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

pub trait BiStream: Send {
    type Reader: AsyncRead + Unpin + Send;
    type Writer: AsyncWrite + Unpin + Send;

    fn split(self) -> (Self::Reader, Self::Writer);
}

pub trait Transport: Send + Sync {
    type Uni: UniStream;
    type Bi: BiStream;

    fn open_uni_stream(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Uni, BoxError>> + Send>>;
    fn accept_uni_stream(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Uni, BoxError>> + Send>>;

    fn open_bi_stream(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Bi, BoxError>> + Send>>;
    fn accept_bi_stream(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Bi, BoxError>> + Send>>;
}
