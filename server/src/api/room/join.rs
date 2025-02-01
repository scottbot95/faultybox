use crate::api::room::auth::{create_token, AuthError};
use crate::api::room::{ClientId, RoomApiState, RoomState};
use axum::extract::State;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use models::room::api::{Claims, JoinRoomInput, JoinRoomOutput};
use models::room::RoomMember;
use std::collections::hash_map::Entry;

/// /api/room/join/
pub async fn join_room(
    jar: CookieJar,
    State(state): State<RoomApiState>,
    Json(input): Json<JoinRoomInput>,
) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let state = state
        .rooms
        .read()
        .await
        .get(&input.room_id)
        .ok_or_else(|| AuthError::NotFound(format!("Room {} not found", input.room_id)))?
        .clone();
    join_room_impl(None, jar, state, input).await
}

pub async fn join_room_impl(
    given_client_id: Option<ClientId>,
    jar: CookieJar,
    state: RoomState,
    input: JoinRoomInput,
) -> Result<(CookieJar, Json<JoinRoomOutput>), AuthError> {
    let JoinRoomInput { room_id, nickname } = input;
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
                vacant.insert(RoomMember::new(client_id.clone(), nickname));
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
    Ok((jar.add(cookie), Json(JoinRoomOutput { room_id, room })))
}
