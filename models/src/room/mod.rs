pub mod api;

use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::GameKind;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub game: GameKind,
}
