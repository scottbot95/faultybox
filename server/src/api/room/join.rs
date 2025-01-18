#![allow(unused_imports)]

use std::collections::hash_map::Entry;
use std::sync::Arc;
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
use crate::api::room::{ClientId, CloseClientFn, RoomApiState};
use crate::api::room::auth::{create_token, AuthError};

pub async fn join_room(
    Path(room_id): Path<RoomId>,
    jar: CookieJar,
    State(state): State<RoomApiState>,
) -> Result<(CookieJar, Json<JoinRoomOutput>),AuthError> {
    let room = state.rooms.read().await
        .get(&room_id)
        .ok_or_else(|| AuthError::NotFound(format!("Room {} not found", room_id)))?
        .clone();

    let client_id: ClientId = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect::<String>()
        .into();

    if let Some(state) = room.write().await.as_mut() {
        match state.clients.entry(client_id.clone()) {
            Entry::Occupied(_) => {
                tracing::error!("Client {} already used", &client_id);
                return Err(AuthError::TokenCreation);
            }
            Entry::Vacant(e) => {
                e.insert(None);
            }
        }
    } else {
        tracing::error!("Attempted to join non-existent room {}", &room_id);
        return Err(AuthError::NotFound(format!("Room {} does not exist", &room_id)));
    }

    let claims = Claims {
        sub: client_id,
        room_id,
    };

    let token = create_token(&claims)?;
    let cookie = Cookie::build(("room_token", token))
        .path("/")
        .http_only(true)
        .build();

    Ok((
        jar.add(cookie),
        Json(JoinRoomOutput { })
    ))
}
