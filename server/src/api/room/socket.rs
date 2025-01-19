use crate::api::room::auth::RoomToken;
use crate::api::room::{ArcLock, RoomApiState, RoomState, ClientState};
use crate::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use models::room::api::{Claims, ClientId};
use models::ws::{ClientMsg, ServerMsg};
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

    if !room.clients.read().await.contains_key(&claims.sub) {
        return Ok((StatusCode::FORBIDDEN, "Invalid token").into_response());
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, claims, room)))
}

async fn handle_socket(
    socket: WebSocket<ServerMsg, ClientMsg>,
    claims: Claims,
    state: RoomState,
) {
    let room_id = claims.room_id;
    let client_id = claims.sub;
    tracing::trace!("Client {} connected to room {}", &client_id, room_id);

    let (mut send, recv) = socket.split();

    let result = send.send(Message::Item(ServerMsg::RoomJoined(
        room_id,
        state.room.read().await.clone(),
    )))
    .await;
    if let Err(e) = result {
        tracing::error!("Error sending message: {}", e);
        return;
    }
    let mut broadcast_rx = state.client_broadcast.subscribe();

    let (control_tx, mut control_rx) = tokio::sync::mpsc::channel(16);
    let broadcast_task = {
        let client_id = client_id.clone();
        spawn(async move {
            while let Ok(msg) = broadcast_rx.recv().await {
                let result = send.send(Message::Item(msg)).await;
                if let Err(e) = result {
                    tracing::warn!("Error sending message to client {}: {}", client_id, e);
                }
            }
        })
    };

    let prev = state
        .clients
        .write().await
        .insert(client_id.clone(), Some(ClientState {
            sender: state.client_broadcast.clone(),
            control: control_tx,
        }));

    if let Some(prev) = prev.flatten() {
        tracing::info!("Client {} reconnected", client_id);
        let _ = prev.control.send(()).await;
    }

    // Listen for client messages
    tokio::select! {
        _ = read_client(client_id.clone(), recv, state.client_messages.clone()) => {
            tracing::trace!("Client {} closed connection", client_id)
        }
        _ = control_rx.recv() => {
            tracing::trace!("Closed old client {} connection", client_id)
        }
    }

    // Client socket closed, cleanup the broadcast task
    broadcast_task.abort();
    let _ = broadcast_task.await;
}

async fn read_client(
    client_id: ClientId,
    mut recv: SplitStream<WebSocket<ServerMsg, ClientMsg>>,
    sender: tokio::sync::mpsc::Sender<(ClientId, ClientMsg)>
) {
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
        sender.send((client_id.clone(), msg)).await.ok();
    }
}
