use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::Error;

pub type FullTrackName = String;
pub type TrackAlias = u64;

#[derive(Default)]
pub struct TrackManager {
    #[allow(dead_code)]
    tracks: RwLock<HashMap<FullTrackName, Arc<TrackState>>>,
    aliases: RwLock<HashMap<TrackAlias, FullTrackName>>,
    request_counter: AtomicU64,
}

#[allow(dead_code)]
struct TrackState {
    name: FullTrackName,
    alias: Option<TrackAlias>,
}

impl TrackManager {
    /// Insert a track if it does not already exist and return a handle to its
    /// state. Existing tracks are returned as-is.
    pub(crate) fn add_track(&self, name: FullTrackName) {
        let mut tracks = self.tracks.write().unwrap();
        tracks.entry(name.clone()).or_insert_with(|| Arc::new(TrackState {
            name,
            alias: None,
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

    /// Generate a new unique request identifier.
    pub fn new_request_id(&self) -> u64 {
        self.request_counter.fetch_add(1, Ordering::SeqCst)
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
                let new = Arc::new(TrackState { name: name.clone(), alias: Some(alias) });
                *entry = new;
            }
        }
        Ok(())
    }

    pub fn resolve_alias(&self, alias: TrackAlias) -> Option<FullTrackName> {
        let aliases = self.aliases.read().unwrap();
        aliases.get(&alias).cloned()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_alias_is_error() {
        let manager = TrackManager::default();
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
        let manager = TrackManager::default();
        manager.add_track("audio".to_string());
        manager.set_track_alias(&"audio".to_string(), 2).unwrap();
        assert_eq!(manager.resolve_alias(2).as_deref(), Some("audio"));
    }

    #[test]
    fn request_id_increments() {
        let manager = TrackManager::default();
        let first = manager.new_request_id();
        let second = manager.new_request_id();
        assert!(second > first);
    }
}
