use crate::api::room::auth::AuthError;
use crate::api::room::{join, RoomApiState};
use axum::extract::{Path, State};
use axum::Json;
use axum_extra::extract::CookieJar;
use models::room::api::JoinRoomOutput;
use models::GameKind;

/// /api/room/create/{gameKind}
pub async fn create_room(
    Path(game_kind): Path<GameKind>,
    jar: CookieJar,
    State(state): State<RoomApiState>,
) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let (room_id, state) = state.create_room(game_kind).await?;
    let leader_id = state.room.read().await.leader.clone();
    join::join_room_impl(Some(leader_id), room_id, jar, state).await
}
