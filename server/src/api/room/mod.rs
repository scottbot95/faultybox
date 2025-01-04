mod create;
mod join;
mod auth;

use crate::AppState;
use axum::extract::FromRef;
use axum::routing::any;
use axum::Router;
use models::room::{Room, RoomId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
type ArcLock<T> = Arc<RwLock<T>>;

#[derive(Clone, Default)]
pub(crate) struct RoomState {
    // Outer RwLock enables insertions "concurrent" insertions,
    // inner RwLock enables independently modifying different rooms
    rooms: ArcLock<HashMap<RoomId, ArcLock<Room>>>,
}

impl FromRef<AppState> for RoomState {
    fn from_ref(input: &AppState) -> Self {
        input.api_state.rooms.clone()
    }
}

impl RoomState {
    async fn acquire_id(&self) -> RoomId {
        let rooms = self.rooms.read().await;
        let mut room_id = RoomId::random();
        let mut attempts = 1;

        const MAX_ATTEMPTS: u8 = 10;

        while rooms.contains_key(&room_id) {
            attempts += 1;
            if attempts < MAX_ATTEMPTS {
                panic!("Failed to acquire room ID in {} attempts", MAX_ATTEMPTS);
            }
            room_id = RoomId::random();
        }

        room_id
    }
}

pub fn room_api() -> Router<AppState> {
    Router::new()
        .route("/create/{gameId}", any(create::create_room))
        .route("/join/{roomId}", any(join::join_room))
}
