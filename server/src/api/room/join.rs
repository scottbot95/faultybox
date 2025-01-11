use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use jsonwebtoken::encode;
use rand::Rng;
use models::room::api::{Claims, JoinRoomOutput};
use models::room::RoomId;
use crate::api::room::{ClientId, RoomApiState};
use crate::api::room::auth::{create_token, AuthError};

pub async fn join_room(
    Path(room_id): Path<RoomId>,
    jar: CookieJar,
    State(state): State<RoomApiState>,
) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let room = state.rooms.read().await.get(&room_id).unwrap().clone();
    let client_id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    if let Some(state) = room.write().await.as_mut() {
        if !state.clients.insert(ClientId(client_id.clone())) {
            tracing::warn!("Client {} already connected", &client_id);
        }
    }

    let claims = Claims {
        sub: client_id,
        room_id,
    };

    let token = create_token(&claims)?;
    let cookie = Cookie::build(("room_token", token))
        .http_only(true)
        .build();

    Ok((
        jar.add(cookie),
        Json(JoinRoomOutput { })
    ))
}
