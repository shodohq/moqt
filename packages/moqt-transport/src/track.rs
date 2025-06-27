use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::Error;

pub type FullTrackName = String;
pub type TrackAlias = u64;

#[derive(Default)]
pub struct TrackManager {
    tracks: RwLock<HashMap<FullTrackName, Arc<TrackState>>>,
    aliases: RwLock<HashMap<TrackAlias, FullTrackName>>,
}

struct TrackState {
    name: FullTrackName,
}

impl TrackManager {
    pub fn assign_alias(&self, alias: TrackAlias, name: FullTrackName) -> Result<(), Error> {
        let mut aliases = self.aliases.write().unwrap();
        if aliases.contains_key(&alias) {
            return Err(Error::DuplicateTrackAlias(alias));
        }
        aliases.insert(alias, name);
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
