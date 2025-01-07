use axum::extract::{Path, State};
use models::room::Room;
use std::sync::Arc;
use axum::Json;
use tokio::sync::RwLock;
use models::GameKind;
use models::room::api::JoinRoomOutput;
use crate::api::room::{join, RoomApiState, RoomState};
use crate::api::room::auth::AuthError;

pub async fn create_room(
    Path(game_id): Path<GameKind>,
    State(state): State<RoomApiState>,
) -> Result<Json<JoinRoomOutput>, AuthError> {
    let room_id = state.acquire_id().await;
    let room = RoomState {
        room: Room {
            game: game_id.clone(),
        },
        clients: Default::default(),
    };
    {
        let rooms = state.rooms.read().await;
        if let Some(room_state) = rooms.get(&room_id) {
            tracing::trace!("Created room {}: {:?}", room_id, room);
            *room_state.write().await = Some(room);
        } else {
            tracing::error!("Acquired room ID didn't create a room");
        }
    }

    join::join_room(Path(room_id), State(state)).await
}