use axum::extract::{Path, State};
use axum::response::Response;
use models::room::Room;
use models::ws::{ClientMsg, ServerMsg};
use std::sync::Arc;
use axum::http::HeaderMap;
use tokio::sync::RwLock;
use crate::api::room::{join, RoomState};
use crate::ws::WebSocketUpgrade;

pub async fn create_room(
    Path(game_id): Path<String>,
    header_map: HeaderMap,
    ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
    State(state): State<RoomState>,
) -> Response {
    let room_id = state.acquire_id().await;
    {
        let room = Room {
            game: game_id.clone(),
        };
        let mut rooms = state.rooms.write().await;
        let entry = rooms
            .entry(room_id.clone())
            .insert_entry(Arc::new(RwLock::new(room)));
        let room = entry.get().read().await;
        tracing::trace!("Created room {}: {:?}", room_id, room);
    }

    join::join_room(Path(room_id), ws, State(state)).await
}