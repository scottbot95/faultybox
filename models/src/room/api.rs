use crate::room::{Room, RoomId};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientId(pub String);

impl ClientId {
    pub fn random() -> Self {
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(12)
            .map(char::from)
            .collect::<String>()
            .into()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<String> for ClientId {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: ClientId,
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomOutput {
    pub room_id: RoomId,
    pub room: Room,
}
