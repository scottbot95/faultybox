use std::collections::hash_map::Entry;
use axum::extract::{Path, State};
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use models::room::api::{Claims, JoinRoomOutput};
use models::room::{RoomId, RoomMember};
use crate::api::room::{ClientId, RoomApiState, RoomState};
use crate::api::room::auth::{create_token, AuthError};

/// /api/room/join/{room_id}
pub async fn join_room(
    Path(room_id): Path<RoomId>,
    jar: CookieJar,
    State(state): State<RoomApiState>,
) -> Result<(CookieJar, Json<JoinRoomOutput>),AuthError> {
    let state = state.rooms.read().await
        .get(&room_id)
        .ok_or_else(|| AuthError::NotFound(format!("Room {} not found", room_id)))?
        .clone();
    join_room_impl(None, room_id, jar, state).await
}

pub async fn join_room_impl(given_client_id: Option<ClientId>, room_id: RoomId, jar: CookieJar, state: RoomState) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let assert_new_client = given_client_id.is_none();
    let client_id = given_client_id.unwrap_or_else(ClientId::random);
    match state.clients.write().await.entry(client_id.clone()) {
        Entry::Occupied(_) => {
            tracing::error!("Client {} already used", &client_id);
            return Err(AuthError::TokenCreation);
        }
        Entry::Vacant(e) => {
            e.insert(None);
        }
    }

    {
        let mut room = state.room.write().await;
        let entry = room.members.entry(client_id.clone());
        match entry {
            Entry::Occupied(_) => {
                if assert_new_client {
                    tracing::warn!("New client {} already in room", client_id);
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(RoomMember::new(client_id.clone()));
            }
        }
    }

    let claims = Claims {
        sub: client_id,
        room_id: room_id.clone(),
    };

    let token = create_token(&claims)?;
    let cookie = Cookie::build(("room_token", token))
        .path("/")
        .http_only(true)
        .build();

    let room = state.room.read().await.clone();
    Ok((
        jar.add(cookie),
        Json(JoinRoomOutput {
            room_id,
            room
        })
    ))
}
