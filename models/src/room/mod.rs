pub mod api;

use crate::room::api::ClientId;
use crate::GameKind;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct RoomId(pub String);

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RoomId {
    pub fn random() -> Self {
        let id: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            // Filter lowercase instead of map to upper to case to not mess with the distribution
            .filter(|c| c.is_ascii_lowercase())
            .take(4)
            .map(char::from)
            .collect();

        RoomId(id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub game: GameKind,
    pub members: HashMap<ClientId, RoomMember>,
    pub leader: ClientId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoomMember {
    pub client_id: ClientId,
    pub nickname: Option<String>,
}

impl RoomMember {
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            nickname: None,
        }
    }
}
