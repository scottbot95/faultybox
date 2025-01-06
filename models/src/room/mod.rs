pub mod api;

use rand::Rng;
use serde::{Deserialize, Serialize};

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
            .take(4)
            .map(char::from)
            .collect();

        RoomId(id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub game: String,
}
