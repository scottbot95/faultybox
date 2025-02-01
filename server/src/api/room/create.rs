use crate::api::room::auth::AuthError;
use crate::api::room::{join, RoomApiState};
use axum::extract::State;
use axum::Json;
use axum_extra::extract::CookieJar;
use models::room::api::{CreateRoomInput, JoinRoomInput, JoinRoomOutput};

/// /api/room/create
pub async fn create_room(
    jar: CookieJar,
    State(state): State<RoomApiState>,
    Json(input): Json<CreateRoomInput>,
) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let nickname = input.nickname.clone();
    let (room_id, state) = state.create_room(input).await?;
    let leader_id = state.room.read().await.leader.clone();
    let join_input = JoinRoomInput { room_id, nickname };
    join::join_room_impl(Some(leader_id), jar, state, join_input).await
}
