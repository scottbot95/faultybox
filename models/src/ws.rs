use crate::room::Room;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    RoomUpdate(Room),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    SetNickname(String),
    Gecko(GeckoClientMsg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeckoClientMsg {}
