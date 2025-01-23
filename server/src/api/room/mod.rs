mod auth;
mod create;
mod join;
mod socket;

use crate::api::room::auth::AuthError;
use crate::AppState;
use axum::extract::FromRef;
use axum::routing::{any, get, post};
use axum::Router;
use models::room::api::ClientId;
use models::room::{Room, RoomId, RoomMember};
use models::ws::{ClientMsg, ServerMsg};
use models::GameKind;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::RwLock;

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
            let dirty = match msg {
                ClientMsg::SetNickname(name) => {
                    if let Some(m) = self.room.write().await.members.get_mut(&client_id) {
                        m.nickname = Some(name);
                    }
                    true
                }
                ClientMsg::Gecko(_) => {
                    todo!()
                }
            };
            if dirty {
                let msg = ServerMsg::RoomUpdate(self.room.read().await.clone());
                if self.client_broadcast.send(msg).is_err() {
                    tracing::error!("Failed to send room update message");
                    return;
                };
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
    async fn create_room(&self, game_kind: GameKind) -> Result<(RoomId, RoomState), AuthError> {
        let leader = ClientId::random();
        let room = Room {
            game: game_kind,
            members: HashMap::from([
                // include the leader
                (leader.clone(), RoomMember::new(leader.clone())),
            ]),
            leader,
        };

        let mut rooms = self.rooms.write().await;
        let mut room_id = RoomId::random();
        let mut attempts = 1;

        const MAX_ATTEMPTS: u8 = 10;
        while rooms.contains_key(&room_id) {
            attempts += 1;
            if attempts < MAX_ATTEMPTS {
                tracing::error!("Failed to acquire room ID in {} attempts", MAX_ATTEMPTS);
                return Err(AuthError::TokenCreation);
            }
            room_id = RoomId::random();
        }
        loop {
            match rooms.entry(room_id.clone()) {
                Occupied(_) => {
                    attempts += 1;
                    if attempts < MAX_ATTEMPTS {
                        panic!("Failed to acquire room ID in {} attempts", MAX_ATTEMPTS);
                    }
                    room_id = RoomId::random();
                }
                Entry::Vacant(e) => {
                    tracing::info!("Created room {}: {:?}", room_id, room);
                    let state = e.insert(RoomState::new(room));
                    return Ok((room_id, state.clone()));
                }
            }
        }
    }
}

pub fn room_api() -> Router<AppState> {
    Router::new()
        .route("/create/{gameId}", post(create::create_room))
        .route("/join/{roomId}", get(join::join_room))
        .route("/connect", any(socket::connect))
}
