mod create;
mod join;
mod auth;
mod socket;

use crate::AppState;
use axum::extract::FromRef;
use axum::routing::{any, get, post};
use axum::Router;
use models::room::{Room, RoomId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
type ArcLock<T> = Arc<RwLock<T>>;

#[derive(Clone, Default)]
pub(crate) struct RoomApiState {
    // Outer RwLock enables insertions "concurrent" insertions,
    // inner RwLock enables independently modifying different rooms
    rooms: ArcLock<HashMap<RoomId, ArcLock<Option<RoomState>>>>,
}

#[derive(Clone, Debug)]
pub struct RoomState {
    room: Room,
    clients: HashSet<ClientId>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ClientId(pub String);

impl AsRef<String> for ClientId {
    fn as_ref(&self) -> &String { &self.0 }
}

impl FromRef<AppState> for RoomApiState {
    fn from_ref(input: &AppState) -> Self {
        input.api_state.rooms.clone()
    }
}

impl RoomApiState {
    async fn acquire_id(&self) -> RoomId {
        let mut rooms = self.rooms.write().await;
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

        rooms.insert(room_id.clone(), Default::default());

        room_id
    }
}

pub fn room_api() -> Router<AppState> {
    Router::new()
        .route("/create/{gameId}", post(create::create_room))
        .route("/join/{roomId}", get(join::join_room))
        .route("/connect", any(socket::connect))
}
