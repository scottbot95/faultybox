use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::encode;
use rand::Rng;
use models::room::api::{Claims, JoinRoomOutput};
use models::room::RoomId;
use crate::api::room::{ClientId, RoomApiState};
use crate::api::room::auth::{create_token, AuthError};

pub async fn join_room(
    Path(room_id): Path<RoomId>,
    State(state): State<RoomApiState>,
) -> Result<Json<JoinRoomOutput>, AuthError> {
    let room = state.rooms.read().await.get(&room_id).unwrap().clone();
    let client_id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    if !room.write().await.clients.insert(ClientId(client_id.clone())) {
        tracing::warn!("Client {} already connected", &client_id);
    }

    let claims = Claims {
        sub: client_id,
        room_id,
    };

    let token = create_token(&claims)?;

    Ok(Json(JoinRoomOutput { token }))
}
