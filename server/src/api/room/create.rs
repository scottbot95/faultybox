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
    let room_id = state.insert_room(Room {
        game: game_kind,
    }).await;

    join::join_room(Path(room_id), jar, State(state)).await
}