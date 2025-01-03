mod gecko;
mod room;

use crate::api::room::room_api;
use crate::{hello, AppState};
use axum::{extract::FromRef, routing::get, Router};

use self::{gecko::gecko_api, room::RoomState};

#[derive(FromRef, Clone, Default)]
pub struct ApiState {
    rooms: RoomState,
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/hello", get(hello))
        .nest("/gecko", gecko_api())
        .nest("/room", room_api())
}
