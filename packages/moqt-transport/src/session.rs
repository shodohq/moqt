use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::{message::ControlMessage, track::TrackManager, transport::Transport};

pub enum State {
    Initializing,
    Active,
    Closing,
}

pub struct Session<T: Transport> {
    state: Arc<Mutex<State>>,
    pub(crate) control_tx: mpsc::Sender<ControlMessage>,
    pub track_manager: TrackManager,
    pub transport: Arc<T>,
}

impl<T: Transport> Session<T> {
    pub fn new(transport: Arc<T>) -> (Self, mpsc::Receiver<ControlMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let session = Session {
            state: Arc::new(Mutex::new(State::Initializing)),
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
}
