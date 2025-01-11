use axum::extract::{Path, State};
use models::room::Room;
use std::sync::Arc;
use axum::Json;
use axum_extra::extract::CookieJar;
use tokio::sync::RwLock;
use models::GameKind;
use models::room::api::JoinRoomOutput;
use crate::api::room::{join, RoomApiState, RoomState};
use crate::api::room::auth::AuthError;

/// /api/room/create/{gameKind}
pub async fn create_room(
    Path(game_kind): Path<GameKind>,
    jar: CookieJar,
    State(state): State<RoomApiState>,
) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let room_id = state.acquire_id().await;
    let room = RoomState {
        room: Room {
            game: game_kind,
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

    join::join_room(Path(room_id), jar, State(state)).await
}