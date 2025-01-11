use serde::{Deserialize, Serialize};
use crate::room::RoomId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub room_id: RoomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomOutput {}
