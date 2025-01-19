use crate::room::{Room, RoomId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    RoomJoined(RoomId, Room),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Gecko(GeckoClientMsg)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeckoClientMsg {

}
