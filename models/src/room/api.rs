use crate::room::RoomId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientId(pub String);

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
pub struct JoinRoomOutput {}
