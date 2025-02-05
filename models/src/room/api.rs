use crate::room::{Room, RoomId};
use crate::GameKind;
use implicit_clone::ImplicitClone;
use rand::Rng;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::str::FromStr;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientId(SmolStr);

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

impl ImplicitClone for ClientId {}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl FromStr for ClientId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: ClientId,
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomInput {
    pub game_kind: GameKind,
    pub nickname: SmolStr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomInput {
    pub room_id: RoomId,
    pub nickname: SmolStr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomOutput {
    pub room_id: RoomId,
    pub room: Room,
}
