mod reducer;

use std::pin::pin;
use futures::channel::oneshot::{Receiver, Sender};
use futures::FutureExt;
use futures::{select, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use yew::{prelude::*};
use yew::platform::spawn_local;
use models::room::{Room, RoomId};
use models::ws::ServerMsg;
use crate::api_client::{use_api, ApiClient, ApiClientImpl};
use crate::pages::room::lobby::reducer::{RoomAction, RoomState};

turf::style_sheet!("src/pages/room/lobby/lobby.scss");

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub room_id: RoomId,
    pub initial_room: Room,
}

#[function_component(RoomLobbyPage)]
pub fn page(props: &Props) -> Html {
    let client = use_api::<ApiClientImpl>();
    let room_state: UseReducerHandle<RoomState> = use_reducer_eq(|| RoomState {
        room_id: props.room_id.clone(),
        room: props.initial_room.clone(),
    });
    {
        let room_state = room_state.clone();
        use_effect_with((), move |_| {
            let ws = client.connect_room().expect("Unable to connect to room");
            // spawn_local(handle_socket(ws, room_state.dispatcher()));
            let handler = SocketHandler {
                dispatcher: room_state.dispatcher()
            };

            handler.spawn(ws)
        });
    }

    html! {
        <div>
            <p>{ format!("Room ID: {}", room_state.room_id) }</p>
            <p>{ format!("Game Kind: {:?}", room_state.room.game) }</p>
            <p>{"Members:"}</p>
            <ul>
                {room_state.room.members.iter().map(|(client_id, member)| html!{
                    <li key={client_id.to_string()}>
                        {"Name:"}
                        {member.nickname.clone().unwrap_or_else(|| "Unknown".to_string())}
                    </li>
                }).collect::<Html>()}
            </ul>
        </div>
    }
}

struct SocketHandler {
    dispatcher: UseReducerDispatcher<RoomState>,
}

impl SocketHandler {
    fn spawn(self, ws: WebSocket) -> SocketTearDown {
        let (control_tx, recv) = futures::channel::oneshot::channel();
        spawn_local(async move {
            self.handle_socket(ws, recv).await;
        });
        SocketTearDown { control_tx }
    }

    async fn handle_socket(self, ws: WebSocket, mut control_rx: Receiver<()>) {
        let (write, mut read) = ws.split();
        let mut fut = pin!(async move {
            while let Some(msg) = read.next().await {
                let server_msg = match msg {
                        Ok(Message::Text(text)) => {
                            log::debug!("WebSocket Received Text: {}", text);
                            serde_json::from_str::<ServerMsg>(&text)
                        }
                        Ok(Message::Bytes(bytes)) => {
                            log::debug!("WebSocket Received Bytes: {:?} (\"{}\")", &bytes, String::from_utf8_lossy(&bytes));
                            serde_json::from_slice(&bytes)
                        }
                        Err(e) => {
                            log::error!("Error: {:?}", e);
                            break;
                        }
                    };
                    if let Ok(msg) = server_msg {
                        self.handle_msg(msg);
                    }
            }
        }.fuse());
        select! {
            _ = fut => {}
            _ = control_rx => {}
        }

        log::info!("Socket read closed");
    }

    fn handle_msg(&self, msg: ServerMsg) {
        log::info!("Server message: {:?}", msg);

        match msg {
            ServerMsg::RoomUpdate(room) => {
                self.dispatcher.dispatch(RoomAction::UpdateRoom(room));
            }
        }
    }
}

struct SocketTearDown {
    control_tx: Sender<()>,
}

impl TearDown for SocketTearDown {
    fn tear_down(self) {
        if self.control_tx.send(()).is_err() {
            log::warn!("Websocket closed already");
        }
    }
}
