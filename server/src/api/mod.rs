mod gecko;
mod room;

use crate::api::room::room_api;
use crate::AppState;
use axum::{extract::FromRef, Router};

use self::{gecko::gecko_api, room::RoomApiState};

#[derive(FromRef, Clone, Default)]
pub struct ApiState {
    rooms: RoomApiState,
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/gecko", gecko_api())
        .nest("/room", room_api())
}
