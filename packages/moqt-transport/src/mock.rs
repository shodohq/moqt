use bytes::Bytes;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::duplex;
use tokio::io::{self, AsyncRead, AsyncWrite, DuplexStream};
use tokio::sync::mpsc;

use crate::transport::{BiStream, BoxError, Transport};

pub struct MockUniStream(DuplexStream);

impl AsyncRead for MockUniStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for MockUniStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, data)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}

impl Unpin for MockUniStream {}

pub struct MockBiStream {
    read: DuplexStream,
    write: DuplexStream,
}

impl BiStream for MockBiStream {
    type Reader = DuplexStream;
    type Writer = DuplexStream;

    fn split(self) -> (Self::Reader, Self::Writer) {
        (self.read, self.write)
    }
}

pub struct MockTransport {
    incoming_unis: mpsc::Receiver<DuplexStream>,
    incoming_bis: mpsc::Receiver<(DuplexStream, DuplexStream)>,
    incoming_datagrams: mpsc::Receiver<Bytes>,

    uni_tx: mpsc::Sender<DuplexStream>,
    bi_tx: mpsc::Sender<(DuplexStream, DuplexStream)>,
    datagram_tx: mpsc::Sender<Bytes>,
}

impl MockTransport {
    pub fn pair() -> (Self, Self) {
        let (uni_tx_a, uni_rx_a) = mpsc::channel(8);
        let (uni_tx_b, uni_rx_b) = mpsc::channel(8);

        let (bi_tx_a, bi_rx_a) = mpsc::channel(8);
        let (bi_tx_b, bi_rx_b) = mpsc::channel(8);

        let (dg_tx_a, dg_rx_a) = mpsc::channel(8);
        let (dg_tx_b, dg_rx_b) = mpsc::channel(8);

        let a = MockTransport {
            incoming_unis: uni_rx_a,
            incoming_bis: bi_rx_a,
            incoming_datagrams: dg_rx_a,
            uni_tx: uni_tx_b,
            bi_tx: bi_tx_b,
            datagram_tx: dg_tx_b,
        };

        let b = MockTransport {
            incoming_unis: uni_rx_b,
            incoming_bis: bi_rx_b,
            incoming_datagrams: dg_rx_b,
            uni_tx: uni_tx_a,
            bi_tx: bi_tx_a,
            datagram_tx: dg_tx_a,
        };

        (a, b)
    }

    pub async fn recv_datagram(&mut self) -> Option<Bytes> {
        self.incoming_datagrams.recv().await
    }
}

#[async_trait::async_trait]
impl Transport for MockTransport {
    type Uni = MockUniStream;
    type Bi = MockBiStream;

    async fn open_uni_stream(&mut self) -> Result<Self::Uni, BoxError> {
        let (local, remote) = duplex(1024);
        self.uni_tx
            .send(remote)
            .await
            .map_err(|e| Box::new(e) as BoxError)?;
        Ok(MockUniStream(local))
    }

    async fn accept_uni_stream(&mut self) -> Result<Self::Uni, BoxError> {
        match self.incoming_unis.recv().await {
            Some(s) => Ok(MockUniStream(s)),
            None => Err("channel closed".into()),
        }
    }

    async fn open_bi_stream(&mut self) -> Result<Self::Bi, BoxError> {
        let (r1, r2) = duplex(1024);
        let (w1, w2) = duplex(1024);
        self.bi_tx
            .send((r2, w2))
            .await
            .map_err(|e| Box::new(e) as BoxError)?;
        Ok(MockBiStream {
            read: r1,
            write: w1,
        })
    }

    async fn accept_bi_stream(&mut self) -> Result<Self::Bi, BoxError> {
        match self.incoming_bis.recv().await {
            Some((r, w)) => Ok(MockBiStream { read: r, write: w }),
            None => Err("channel closed".into()),
        }
    }

    async fn send_datagram(&mut self, data: Bytes) -> Result<(), BoxError> {
        self.datagram_tx
            .send(data)
            .await
            .map_err(|e| Box::new(e) as BoxError)
    }
}
