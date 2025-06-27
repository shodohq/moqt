use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::{
    error::Error,
    message::{ControlMessage, Goaway},
    track::TrackManager,
    transport::Transport,
};

pub enum State {
    Initializing,
    Active,
    Closing,
}

pub struct Session<T: Transport> {
    state: Arc<Mutex<State>>,
    received_goaway: Arc<Mutex<bool>>,
    pub(crate) control_tx: mpsc::Sender<ControlMessage>,
    pub track_manager: TrackManager,
    pub transport: Arc<T>,
}

impl<T: Transport> Session<T> {
    pub fn new(transport: Arc<T>) -> (Self, mpsc::Receiver<ControlMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let session = Session {
            state: Arc::new(Mutex::new(State::Initializing)),
            received_goaway: Arc::new(Mutex::new(false)),
            control_tx: tx,
            track_manager: TrackManager::default(),
            transport,
        };
        (session, rx)
    }

    pub async fn send_control(&self, msg: ControlMessage) -> Result<(), crate::error::Error> {
        self.control_tx
            .send(msg)
            .await
            .map_err(|e| crate::error::Error::Transport(Box::new(e)))
    }

    /// Process an incoming GOAWAY message. `is_server` indicates whether this
    /// endpoint is acting as a server when receiving the message.
    pub fn handle_goaway(&self, msg: &Goaway, is_server: bool) -> Result<(), Error> {
        {
            let mut received = self.received_goaway.lock().unwrap();
            if *received {
                return Err(Error::ProtocolViolation {
                    reason: "multiple GOAWAY messages".into(),
                });
            }
            *received = true;
        }

        if is_server && msg.new_session_uri.is_some() {
            return Err(Error::ProtocolViolation {
                reason: "GOAWAY from client contained URI".into(),
            });
        }

        let mut state = self.state.lock().unwrap();
        *state = State::Closing;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
    use crate::transport::{BiStream, BoxError};

    #[derive(Clone)]
    struct DummyStream;

    impl AsyncRead for DummyStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for DummyStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Ok(0))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    struct DummyBi;

    impl BiStream for DummyBi {
        type Reader = DummyStream;
        type Writer = DummyStream;

        fn split(self) -> (Self::Reader, Self::Writer) {
            (DummyStream, DummyStream)
        }
    }

    #[derive(Clone)]
    struct DummyTransport;

    impl Transport for DummyTransport {
        type Uni = DummyStream;
        type Bi = DummyBi;

        fn open_uni_stream(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Uni, BoxError>> + Send>> {
            Box::pin(async { unimplemented!() })
        }

        fn accept_uni_stream(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Uni, BoxError>> + Send>> {
            Box::pin(async { unimplemented!() })
        }

        fn open_bi_stream(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Bi, BoxError>> + Send>> {
            Box::pin(async { unimplemented!() })
        }

        fn accept_bi_stream(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Bi, BoxError>> + Send>> {
            Box::pin(async { unimplemented!() })
        }
    }

    #[test]
    fn multiple_goaway_is_violation() {
        let (session, _rx) = Session::new(Arc::new(DummyTransport));

        session
            .handle_goaway(&Goaway { new_session_uri: None }, false)
            .unwrap();
        let err = session
            .handle_goaway(&Goaway { new_session_uri: None }, false)
            .unwrap_err();
        match err {
            Error::ProtocolViolation { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn server_rejects_uri() {
        let (session, _rx) = Session::new(Arc::new(DummyTransport));

        let err = session
            .handle_goaway(
                &Goaway {
                    new_session_uri: Some("https://example.com".into()),
                },
                true,
            )
            .unwrap_err();

        match err {
            Error::ProtocolViolation { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn server_accepts_no_uri() {
        let (session, _rx) = Session::new(Arc::new(DummyTransport));
        session
            .handle_goaway(&Goaway { new_session_uri: None }, true)
            .unwrap();
    }

    #[test]
    fn client_accepts_uri_and_sets_state() {
        let (session, _rx) = Session::new(Arc::new(DummyTransport));

        session
            .handle_goaway(
                &Goaway {
                    new_session_uri: Some("https://example.com".into()),
                },
                false,
            )
            .unwrap();

        let state = session.state.lock().unwrap();
        match *state {
            State::Closing => {}
            _ => panic!("unexpected state"),
        }
    }
}
