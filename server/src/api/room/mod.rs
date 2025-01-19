mod auth;
mod create;
mod join;
mod socket;

use crate::AppState;
use axum::extract::FromRef;
use axum::routing::{any, get, post};
use axum::Router;
use models::room::api::ClientId;
use models::room::{Room, RoomId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::RwLock;
use models::ws::{ClientMsg, ServerMsg};

type ArcLock<T> = Arc<RwLock<T>>;

#[derive(Clone, Default)]
pub(crate) struct RoomApiState {
    // Outer RwLock enables insertions "concurrent" insertions,
    // inner RwLock enables independently modifying different rooms
    rooms: ArcLock<HashMap<RoomId, RoomState>>,
}

#[derive(Clone)]
pub struct ClientState {
    /// Messages sent to this channel will be sent to the client
    sender: tokio::sync::broadcast::Sender<ServerMsg>,
    /// Sending any message to this channel will trigger the client socket to be closed
    control: tokio::sync::mpsc::Sender<()>,
}

#[derive(Clone)]
pub struct RoomState {
    room: ArcLock<Room>,
    clients: ArcLock<HashMap<ClientId, Option<ClientState>>>,
    client_broadcast: tokio::sync::broadcast::Sender<ServerMsg>,
    client_messages: tokio::sync::mpsc::Sender<(ClientId, ClientMsg)>,
}

impl RoomState {
    pub fn new(room: Room) -> Self {
        let (client_broadcast, _) = tokio::sync::broadcast::channel(16);
        let (client_tx, client_rx) = tokio::sync::mpsc::channel(16);
        let this = Self {
            room: Arc::new(RwLock::new(room)),
            clients: Default::default(),
            client_broadcast,
            client_messages: client_tx,
        };

        spawn(this.clone().listen(client_rx));

        this
    }
}

impl RoomState {
    async fn listen(self, mut rx: tokio::sync::mpsc::Receiver<(ClientId, ClientMsg)>) {
        while let Some((client_id, msg)) = rx.recv().await {
            match msg {
                ClientMsg::Gecko(_) => {
                    todo!()
                }
            }
        }
    }
}

impl Debug for RoomState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomState")
            .field("room", &self.room)
            .field("clients", &self.clients.blocking_read().keys())
            .finish()
    }
}

impl FromRef<AppState> for RoomApiState {
    fn from_ref(input: &AppState) -> Self {
        input.api_state.rooms.clone()
    }
}

impl RoomApiState {
    async fn insert_room(&self, room: Room) -> RoomId {
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

        tracing::info!("Created room {}: {:?}", room_id, room);
        rooms.insert(room_id.clone(), RoomState::new(room));

        room_id
    }
}

pub fn room_api() -> Router<AppState> {
    Router::new()
        .route("/create/{gameId}", post(create::create_room))
        .route("/join/{roomId}", get(join::join_room))
        .route("/connect", any(socket::connect))
}
