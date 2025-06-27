use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;

use crate::message::{ControlMessage, Subscribe};

use crate::error::Error;

pub type FullTrackName = String;
pub type TrackAlias = u64;

pub struct TrackManager {
    #[allow(dead_code)]
    tracks: RwLock<HashMap<FullTrackName, Arc<TrackState>>>,
    aliases: RwLock<HashMap<TrackAlias, FullTrackName>>,
    requests: RwLock<HashMap<u64, FullTrackName>>,
    next_request_id: AtomicU64,
    control_tx: mpsc::Sender<ControlMessage>,
}

#[allow(dead_code)]
struct TrackState {
    name: FullTrackName,
    alias: Option<TrackAlias>,
    subscribers: Vec<mpsc::Sender<Result<Object, Error>>>,
}

impl TrackManager {
    pub fn new(control_tx: mpsc::Sender<ControlMessage>) -> Self {
        TrackManager {
            tracks: RwLock::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
            requests: RwLock::new(HashMap::new()),
            next_request_id: AtomicU64::new(0),
            control_tx,
        }
    }

    /// Insert a track if it does not already exist and return a handle to its
    /// state. Existing tracks are returned as-is.
    pub(crate) fn add_track(&self, name: FullTrackName) {
        let mut tracks = self.tracks.write().unwrap();
        tracks.entry(name.clone()).or_insert_with(|| Arc::new(TrackState {
            name,
            alias: None,
            subscribers: Vec::new(),
        }));
    }

    pub fn assign_alias(&self, alias: TrackAlias, name: FullTrackName) -> Result<(), Error> {
        let mut aliases = self.aliases.write().unwrap();
        if aliases.contains_key(&alias) {
            return Err(Error::DuplicateTrackAlias(alias));
        }
        aliases.insert(alias, name);
        Ok(())
    }

    /// Associate an alias with an existing track. Returns an error on
    /// duplication.
    pub(crate) fn set_track_alias(&self, name: &FullTrackName, alias: TrackAlias) -> Result<(), Error> {
        self.assign_alias(alias, name.clone())?;
        if let Some(entry) = self.tracks.write().unwrap().get_mut(name) {
            if let Some(state) = Arc::get_mut(entry) {
                state.alias = Some(alias);
            } else {
                // There are outstanding references; replace with updated copy.
                let new = Arc::new(TrackState {
                    name: name.clone(),
                    alias: Some(alias),
                    subscribers: Vec::new(),
                });
                *entry = new;
            }
        }
        Ok(())
    }

    pub fn resolve_alias(&self, alias: TrackAlias) -> Option<FullTrackName> {
        let aliases = self.aliases.read().unwrap();
        aliases.get(&alias).cloned()
    }

    /// Start a new subscription for the given track name.
    pub async fn subscribe(&self, name: FullTrackName) -> Result<ObjectStream, Error> {
        self.add_track(name.clone());

        let req_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
        self.requests.write().unwrap().insert(req_id, name.clone());

        let (tx, rx) = mpsc::channel(16);
        if let Some(entry) = self.tracks.write().unwrap().get_mut(&name) {
            if let Some(state) = Arc::get_mut(entry) {
                state.subscribers.push(tx.clone());
            }
        }

        let msg = Subscribe {
            request_id: req_id,
            track_namespace: 0,
            track_name: name,
            subscriber_priority: 0,
            group_order: 0,
            forward: 1,
            filter_type: 0x2,
            start_location: None,
            end_group: None,
            parameters: Vec::new(),
        };

        self.control_tx
            .send(ControlMessage::Subscribe(msg))
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        Ok(ObjectStream { rx })
    }

    pub async fn handle_subscribe_ok(&self, ok: crate::message::SubscribeOk) -> Result<(), Error> {
        if let Some(name) = self.requests.write().unwrap().remove(&ok.request_id) {
            self.set_track_alias(&name, ok.track_alias)?;
        }
        Ok(())
    }
}

pub struct Track {
    pub name: FullTrackName,
}

pub struct TrackPublisher {
    track_alias: TrackAlias,
}

impl TrackPublisher {
    pub fn alias(&self) -> TrackAlias {
        self.track_alias
    }
}

pub struct ObjectStream {
    pub(crate) rx: mpsc::Receiver<Result<Object, Error>>,
}

impl futures::Stream for ObjectStream {
    type Item = Result<Object, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.rx).poll_recv(cx)
    }
}

pub struct Object {
    pub metadata: ObjectMetadata,
    pub payload: Bytes,
}

pub struct ObjectMetadata {
    pub track_alias: u64,
    pub group_id: u64,
    pub object_id: u64,
    pub priority: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_alias_is_error() {
        let (tx, _rx) = mpsc::channel(1);
        let manager = TrackManager::new(tx);
        manager.add_track("video".to_string());
        assert!(manager.set_track_alias(&"video".to_string(), 1).is_ok());
        let err = manager.set_track_alias(&"video".to_string(), 1).unwrap_err();
        match err {
            Error::DuplicateTrackAlias(1) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn resolve_returns_name() {
        let (tx, _rx) = mpsc::channel(1);
        let manager = TrackManager::new(tx);
        manager.add_track("audio".to_string());
        manager.set_track_alias(&"audio".to_string(), 2).unwrap();
        assert_eq!(manager.resolve_alias(2).as_deref(), Some("audio"));
    }
}
