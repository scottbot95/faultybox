use crate::room::Room;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    RoomUpdate(Room),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    SetNickname(SmolStr),
    Gecko(GeckoClientMsg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeckoClientMsg {}
