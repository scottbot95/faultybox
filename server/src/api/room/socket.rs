use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use models::room::{Room, RoomId};
use models::room::api::Claims;
use models::ws::{ClientMsg, ServerMsg};
use crate::api::room::{ArcLock, ClientId, RoomApiState, RoomState};
use crate::api::room::auth::RoomToken;
use crate::ws::{Message, WebSocket, WebSocketUpgrade};

pub async fn connect(
    RoomToken(claims): RoomToken,
    ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
    State(state): State<RoomApiState>,
) -> Result<Response, Response> {
    let room = state.rooms.read()
        .await
        .get(&claims.room_id)
        .ok_or_else(|| StatusCode::FORBIDDEN.into_response())?
        .clone();

    if let Some(room) = room.read().await.as_ref() {
        if !room.clients.contains(&ClientId(claims.sub.clone())) {
            return Ok((StatusCode::FORBIDDEN, "Invalid token").into_response());
        }
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, claims, room)))
}

async fn handle_socket(
    mut socket: WebSocket<ServerMsg, ClientMsg>,
    claims: Claims,
    room: ArcLock<Option<RoomState>>,
) {
    let room_id = claims.room_id;
    tracing::trace!("Client {} connected to room {}", &claims.sub, room_id);

    if let Some(state) = room.read().await.as_ref() {
        socket
            .send_item(ServerMsg::RoomJoined(room_id, state.room.clone()))
            .await
            .expect("Failed to send room joined");
    }

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(Message::Item(msg)) => msg,
            Ok(_) => continue,
            Err(err) => {
                tracing::error!("got error: {}", err);
                continue;
            }
        };

        tracing::trace!("Message received: {:?}", msg);
    }

    tracing::trace!("Client {} disconnected", claims.sub);
}