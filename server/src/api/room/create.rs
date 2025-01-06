use axum::extract::{Path, State};
use models::room::Room;
use std::sync::Arc;
use axum::Json;
use tokio::sync::RwLock;
use models::room::api::JoinRoomOutput;
use crate::api::room::{join, RoomApiState, RoomState};
use crate::api::room::auth::AuthError;

pub async fn create_room(
    Path(game_id): Path<String>,
    State(state): State<RoomApiState>,
) -> Result<Json<JoinRoomOutput>, AuthError> {
    let room_id = state.acquire_id().await;
    {
        let room = RoomState {
            room: Room {
                game: game_id.clone(),
            },
            clients: Default::default(),
        };
        let mut rooms = state.rooms.write().await;
        let entry = rooms
            .entry(room_id.clone())
            .insert_entry(Arc::new(RwLock::new(room)));
        let room = entry.get().read().await;
        tracing::trace!("Created room {}: {:?}", room_id, room);
    }

    join::join_room(Path(room_id), State(state)).await
}