use crate::AppState;
use axum::extract::{FromRef, Path};
use axum::response::IntoResponse;
use axum::routing::{any, post};
use axum::{extract::State, Router};
// use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use models::room::{Room, RoomId};
use models::ws::{ClientMsg, ServerMsg};
use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::Message;
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

pub(super) fn room_api() -> Router<AppState> {
    Router::new()
        .route("/create/:gameId", any(create_room))
        .route("/join/:roomId", any(join_room))
}

async fn create_room(
    Path(game_id): Path<String>,
    // ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<RoomState>,
) -> impl IntoResponse {
    let room_id = state.acquire_id().await;
    {
        let room = Room {
            game: game_id.clone(),
        };
        let mut rooms = state.rooms.write().await;
        // rooms.insert(room_id.clone(), Arc::new(RwLock::new(room)));
        let entry = rooms
            .entry(room_id.clone())
            .insert_entry(Arc::new(RwLock::new(room)));
        let room = entry.get().read().await;
        tracing::trace!("Created room {}: {:?}", room_id, room);
    }

    join_room(Path(room_id), ws, State(state)).await;
}

async fn join_room(
    Path(room_id): Path<RoomId>,
    // ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<RoomState>,
) -> impl IntoResponse {
    let room = state.rooms.read().await.get(&room_id).unwrap().clone();
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, room))
}

#[allow(unused)]
async fn handle_socket(
    // mut socket: WebSocket<ServerMsg, ClientMsg>,
    mut socket: axum::extract::ws::WebSocket,
    room_id: RoomId,
    room: ArcLock<Room>,
) {
    tracing::trace!("Joined room {}", room_id);

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(Message::Text(msg)) => msg,
            Ok(_) => continue,
            Err(err) => {
                tracing::error!("got error: {}", err);
                continue;
            }
        };
        
        tracing::trace!("Message received: {:?}", msg);
    }
    
    tracing::trace!("Websocket closed");
}
