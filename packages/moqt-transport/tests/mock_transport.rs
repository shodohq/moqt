use moqt_transport::mock::MockTransport;
use moqt_transport::transport::{Transport, BiStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::Bytes;

#[test]
fn unidirectional_stream_roundtrip() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let (mut a, mut b) = MockTransport::pair();

        let mut send = a.open_uni_stream().await.unwrap();
        let mut recv = b.accept_uni_stream().await.unwrap();

        send.write_all(b"hello").await.unwrap();
        send.shutdown().await.unwrap();

        let mut buf = Vec::new();
        recv.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"hello");
    });
}

#[test]
fn bidirectional_stream_roundtrip() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let (mut a, mut b) = MockTransport::pair();

        let mut client = a.open_bi_stream().await.unwrap();
        let mut server = b.accept_bi_stream().await.unwrap();

        let (mut cr, mut cw) = client.split();
        let (mut sr, mut sw) = server.split();

        cw.write_all(b"ping").await.unwrap();
        cw.shutdown().await.unwrap();
        let mut buf = Vec::new();
        sr.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"ping");

        sw.write_all(b"pong").await.unwrap();
        sw.shutdown().await.unwrap();
        let mut buf2 = Vec::new();
        cr.read_to_end(&mut buf2).await.unwrap();
        assert_eq!(buf2, b"pong");
    });
}

#[test]
fn datagram_send_recv() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let (mut a, mut b) = MockTransport::pair();
        a.send_datagram(Bytes::from_static(b"data")).await.unwrap();
        let d = b.recv_datagram().await.unwrap();
        assert_eq!(d, Bytes::from_static(b"data"));
    });
}
