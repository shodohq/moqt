use bytes::Bytes;
use futures_core::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use tokio::sync::mpsc;

use crate::error::Error;
use crate::message::SubscribeOk;

pub type FullTrackName = String;
pub type TrackAlias = u64;

pub struct TrackManager {
    #[allow(dead_code)]
    tracks: RwLock<HashMap<FullTrackName, Arc<std::sync::Mutex<TrackState>>>>,
    aliases: RwLock<HashMap<TrackAlias, FullTrackName>>,
    requests: RwLock<HashMap<u64, FullTrackName>>,
    request_counter: AtomicU64,
    max_request_id: AtomicU64,
}

impl Default for TrackManager {
    fn default() -> Self {
        Self {
            tracks: RwLock::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
            requests: RwLock::new(HashMap::new()),
            request_counter: AtomicU64::new(0),
            max_request_id: AtomicU64::new(0),
        }
    }
}

#[allow(dead_code)]
struct TrackState {
    name: FullTrackName,
    alias: Option<TrackAlias>,
    subscribers: Vec<mpsc::Sender<Result<Object, Error>>>,
}

impl TrackManager {
    /// Insert a track if it does not already exist and return a handle to its
    /// state. Existing tracks are returned as-is.
    pub(crate) fn add_track(&self, name: FullTrackName) {
        let mut tracks = self.tracks.write().unwrap();
        tracks.entry(name.clone()).or_insert_with(|| {
            Arc::new(std::sync::Mutex::new(TrackState {
                name,
                alias: None,
                subscribers: Vec::new(),
            }))
        });
    }

    pub fn assign_alias(&self, alias: TrackAlias, name: FullTrackName) -> Result<(), Error> {
        let mut aliases = self.aliases.write().unwrap();
        if aliases.contains_key(&alias) {
            return Err(Error::DuplicateTrackAlias(alias));
        }
        aliases.insert(alias, name);
        Ok(())
    }

    /// Generate a new unique request identifier. Returns an error if the peer
    /// has not allowed opening additional requests.
    pub fn new_request_id(&self) -> Result<u64, Error> {
        let next = self.request_counter.load(Ordering::SeqCst);
        let max = self.max_request_id.load(Ordering::SeqCst);
        if next >= max {
            return Err(Error::TooManyRequests);
        }
        Ok(self.request_counter.fetch_add(1, Ordering::SeqCst))
    }

    /// Associate an alias with an existing track. Returns an error on
    /// duplication.
    pub(crate) fn set_track_alias(
        &self,
        name: &FullTrackName,
        alias: TrackAlias,
    ) -> Result<(), Error> {
        self.assign_alias(alias, name.clone())?;
        if let Some(entry) = self.tracks.write().unwrap().get_mut(name) {
            let mut state = entry.lock().unwrap();
            state.alias = Some(alias);
        }
        Ok(())
    }

    pub fn resolve_alias(&self, alias: TrackAlias) -> Option<FullTrackName> {
        let aliases = self.aliases.read().unwrap();
        aliases.get(&alias).cloned()
    }

    /// Update the maximum request ID permitted by the peer. The provided value
    /// MUST be strictly greater than any previously received value.
    pub fn handle_max_request_id(&self, new_max: u64) -> Result<(), Error> {
        let current = self.max_request_id.load(Ordering::SeqCst);
        if new_max <= current {
            return Err(Error::ProtocolViolation {
                reason: "MAX_REQUEST_ID decreased".into(),
            });
        }
        self.max_request_id.store(new_max, Ordering::SeqCst);
        Ok(())
    }

    /// Start a new subscription to the given track name. Returns the request id and a stream of objects.
    pub fn subscribe_track(&self, name: FullTrackName) -> Result<(u64, ObjectStream), Error> {
        self.add_track(name.clone());
        let request_id = self.new_request_id()?;
        let (tx, rx) = mpsc::channel(16);

        if let Some(entry) = self.tracks.read().unwrap().get(&name) {
            let mut state = entry.lock().unwrap();
            state.subscribers.push(tx);
        }

        self.requests.write().unwrap().insert(request_id, name);
        Ok((request_id, ObjectStream { rx }))
    }

    /// Process SUBSCRIBE_OK by registering the alias and clearing pending state.
    pub fn handle_subscribe_ok(&self, ok: &SubscribeOk) -> Result<(), Error> {
        let name = {
            let mut reqs = self.requests.write().unwrap();
            reqs.remove(&ok.request_id)
        };
        let name = name.ok_or_else(|| Error::ProtocolViolation {
            reason: "unknown request".into(),
        })?;
        self.set_track_alias(&name, ok.track_alias)
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

/// Stream of objects for a subscription.
pub struct ObjectStream {
    rx: mpsc::Receiver<Result<Object, Error>>,
}

impl Stream for ObjectStream {
    type Item = Result<Object, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_alias_is_error() {
        let manager = TrackManager::default();
        manager.add_track("video".to_string());
        assert!(manager.set_track_alias(&"video".to_string(), 1).is_ok());
        let err = manager
            .set_track_alias(&"video".to_string(), 1)
            .unwrap_err();
        match err {
            Error::DuplicateTrackAlias(1) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn resolve_returns_name() {
        let manager = TrackManager::default();
        manager.add_track("audio".to_string());
        manager.set_track_alias(&"audio".to_string(), 2).unwrap();
        assert_eq!(manager.resolve_alias(2).as_deref(), Some("audio"));
    }

    #[test]
    fn request_id_increments() {
        let manager = TrackManager::default();
        manager.handle_max_request_id(10).unwrap();
        let first = manager.new_request_id().unwrap();
        let second = manager.new_request_id().unwrap();
        assert!(second > first);
    }

    #[test]
    fn subscribe_creates_mapping() {
        let manager = TrackManager::default();
        manager.handle_max_request_id(10).unwrap();
        let (id, stream) = manager.subscribe_track("video".to_string()).unwrap();
        assert_eq!(
            manager.requests.read().unwrap().get(&id),
            Some(&"video".to_string())
        );
        drop(stream);
    }

    #[test]
    fn handle_subscribe_ok_sets_alias() {
        let manager = TrackManager::default();
        manager.handle_max_request_id(10).unwrap();
        let (id, _stream) = manager.subscribe_track("audio".to_string()).unwrap();
        let ok = SubscribeOk {
            request_id: id,
            track_alias: 7,
            expires: 0,
            group_order: 1,
            content_exists: false,
            largest_location: None,
            parameters: Vec::new(),
        };
        manager.handle_subscribe_ok(&ok).unwrap();
        assert_eq!(manager.resolve_alias(7).as_deref(), Some("audio"));
    }

    #[test]
    fn max_request_id_must_increase() {
        let manager = TrackManager::default();
        manager.handle_max_request_id(10).unwrap();
        let err = manager.handle_max_request_id(5).unwrap_err();
        match err {
            Error::ProtocolViolation { .. } => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn new_request_id_respects_limit() {
        let manager = TrackManager::default();
        manager.handle_max_request_id(1).unwrap();
        let _ = manager.new_request_id().unwrap();
        let err = manager.new_request_id().unwrap_err();
        match err {
            Error::TooManyRequests => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
