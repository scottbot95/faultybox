use crate::api::room::auth::RoomToken;
use crate::api::room::{ClientState, RoomApiState, RoomState};
use crate::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use models::room::api::{Claims, ClientId};
use models::ws::{ClientMsg, ServerMsg};
use std::time::Duration;
use tokio::spawn;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;

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
        .ok_or_else(|| (StatusCode::FORBIDDEN, "Room has ended").into_response())?
        .clone();

    if !room.clients.read().await.contains_key(&claims.sub) {
        return Ok((StatusCode::FORBIDDEN, "Invalid token").into_response());
    }

    let resp = ws
        .on_failed_upgrade(|e| {
            tracing::warn!("Failed to establish websocket: {}", e);
        })
        .on_upgrade(move |socket| handle_socket(socket, claims, room));
    Ok(resp)
}

async fn handle_socket(socket: WebSocket<ServerMsg, ClientMsg>, claims: Claims, state: RoomState) {
    let room_id = claims.room_id;
    let client_id = claims.sub;
    tracing::trace!("Client {} connected to room {}", &client_id, room_id);

    let (mut send, recv) = socket.split();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(10);
    let mut broadcast_rx = state.client_broadcast.subscribe();

    let (control_tx, mut control_rx) = tokio::sync::mpsc::channel(1);

    let broadcast_task = {
        let client_id = client_id.clone();
        let sender = sender.clone();
        spawn(async move {
            loop {
                tokio::select! {
                    next = broadcast_rx.recv() => match next {
                        Ok(msg) => {
                            if sender.send(msg).await.is_err() {
                                tracing::warn!(
                                    "Could not forward broadcast for {}. Client receiver channel closed",
                                    client_id
                                )
                            }
                        }
                        Err(RecvError::Lagged(n)) => {
                            tracing::warn!("Broadcast for {} lagging. Missed {} message", client_id, n);
                        }
                        Err(RecvError::Closed) => {
                            break;
                        }
                    },

                }
            }
            tracing::warn!(%client_id, "Broadcast forwarder closed early");
        })
    };

    let socket_send_task = {
        let room_id = room_id.clone();
        let client_id = client_id.clone();
        spawn(async move {
            loop {
                tokio::select! {
                    msg = receiver.recv() => {
                        if let Some(msg) = msg {
                            let result = send.send(Message::Item(msg)).await;
                            if let Err(e) = result {
                                tracing::warn!("Error sending message to client {}: {}", client_id, e);
                            }
                        } else {
                            tracing::warn!("Receiver channel closed for client: {}", client_id);
                            break;
                        }
                    }
                    _ = sleep(Duration::from_secs(55)) => {
                        // It's been a while since we sent anything,
                        // send a Ping to keep the connection alive
                        tracing::trace!("Heartbeating socket for {} in room {}", client_id, room_id);
                        let result = send.send(Message::Ping(Bytes::new())).await;
                        if let Err(e) = result {
                            tracing::warn!("Error sending message to client {}: {}", client_id, e);
                        }
                    }
                }
            }
            tracing::warn!(%client_id, %room_id, "WebSocket sender closed early");
        })
    };

    let prev = state.clients.write().await.insert(
        client_id.clone(),
        Some(ClientState {
            sender,
            control: control_tx,
        }),
    );

    if let Some(prev) = prev.flatten() {
        tracing::info!("Client {} reconnected", client_id);
        let _ = prev.control.send(()).await;
    }

    {
        let room = state.room.read().await.clone();
        let msg = ServerMsg::RoomUpdate(room);
        if state.client_broadcast.send(msg).is_err() {
            tracing::error!(%client_id, %room_id, "No receivers on room broadcast channel");
        }
    }

    // Listen for client messages
    tokio::select! {
        _ = read_client(client_id.clone(), recv, state.client_messages.clone()) => {
            tracing::trace!("Client {} closed connection", client_id);
        }
        _ = control_rx.recv() => {
            tracing::trace!("Client {} received close signal", client_id);
            control_rx.close();
        }
    }

    tracing::trace!(%client_id, %room_id, "Closing Socket");

    // Client socket closed, cleanup the broadcast task
    broadcast_task.abort();
    socket_send_task.abort();

    let _ = tokio::join!(broadcast_task, socket_send_task);

    tracing::trace!("Socket closed: {}", client_id);
}

async fn read_client(
    client_id: ClientId,
    mut recv: SplitStream<WebSocket<ServerMsg, ClientMsg>>,
    sender: tokio::sync::mpsc::Sender<(ClientId, ClientMsg)>,
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
