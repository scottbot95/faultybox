use crate::api::room::auth::RoomToken;
use crate::api::room::{CloseClientFn, ArcLock, RoomApiState, RoomState};
use crate::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use models::room::api::Claims;
use models::ws::{ClientMsg, ServerMsg};
use std::sync::Arc;
use tokio::spawn;

pub async fn connect(
    RoomToken(claims): RoomToken,
    ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
    State(state): State<RoomApiState>,
) -> Result<Response, Response> {
    let room = state
        .rooms
        .read()
        .await
        .get(&claims.room_id)
        .ok_or_else(|| StatusCode::FORBIDDEN.into_response())?
        .clone();

    if let Some(room) = room.read().await.as_ref() {
        if !room.clients.contains_key(&claims.sub) {
            return Ok((StatusCode::FORBIDDEN, "Invalid token").into_response());
        }
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, claims, room)))
}

async fn handle_socket(
    socket: WebSocket<ServerMsg, ClientMsg>,
    claims: Claims,
    room: ArcLock<Option<RoomState>>,
) {
    let room_id = claims.room_id;
    tracing::trace!("Client {} connected to room {}", &claims.sub, room_id);

    let (mut send, recv) = socket.split();

    if let Some(state) = room.read().await.as_ref() {
        send.send(Message::Item(ServerMsg::RoomJoined(
            room_id,
            state.room.clone(),
        )))
        .await
        .expect("Failed to send room joined");
    }

    fn make_abort<F: Fn() +Send+Sync + 'static>(f: F) -> Arc<CloseClientFn> {
        Arc::new(Box::new(f))
    }
    let abort_fn = {
        let sub = claims.sub.clone();
        let handle = spawn(read_client(recv));
        make_abort(move || {
            handle.abort();
            tracing::trace!("Closing client {}", sub);
        })
    };
    if let Some(state) = &mut *room.write().await {
        let prev = state
            .clients
            .insert(claims.sub.clone(), Some(abort_fn));

        if let Some(prev) = prev.flatten() {
            tracing::info!("Client {} reconnected", claims.sub);
            (*prev)();
        }
    }
}

async fn read_client(mut recv: SplitStream<WebSocket<ServerMsg, ClientMsg>>) {
    while let Some(msg) = recv.next().await {
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
}
