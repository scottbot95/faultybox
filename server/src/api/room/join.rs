use axum::extract::{Path, State};
use axum::response::Response;
use models::room::{Room, RoomId};
use models::ws::{ClientMsg, ServerMsg};
use crate::api::room::{ArcLock, RoomState};
use crate::ws::{Message, WebSocket, WebSocketUpgrade};

pub async fn join_room(
    Path(room_id): Path<RoomId>,
    ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
    State(state): State<RoomState>,
) -> Response {
    let room = state.rooms.read().await.get(&room_id).unwrap().clone();
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, room))
}

#[allow(unused)]
async fn handle_socket(
    mut socket: WebSocket<ServerMsg, ClientMsg>,
    room_id: RoomId,
    room: ArcLock<Room>,
) {
    tracing::trace!("Joined room {}", room_id);
    
    socket.send_item(ServerMsg::RoomJoined(room_id, room.read().await.clone())).await;

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
    
    tracing::trace!("Websocket closed");
}