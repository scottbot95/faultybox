use std::{error::Error, rc::Rc};

use futures::{channel::oneshot::{Receiver, Sender}, FutureExt, StreamExt, TryFutureExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use models::{room::RoomId, ws::ServerMsg, GameKind};
use yew::{platform::spawn_local, prelude::*};
use futures::future::select;
use futures::future::Either;

use crate::api_client::{use_api, ApiClient, ApiClientImpl};

turf::style_sheet!("src/pages/room/lobby.scss");

#[function_component(RoomLobbyPage)]
pub fn page() -> Html {
    let client = use_api::<ApiClientImpl>();
    let room_state = use_reducer_eq(RoomState::default);
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

    let body = match &*room_state {
        RoomState { room_id: Some(room_id), game_kind: Some(game_kind) } => html! {
            <div>
                <p>{ format!("Room ID: {}", room_id) }</p>
                <p>{ format!("Game Kind: {:?}", game_kind) }</p>
            </div>
        },
        _ => html! {
            <div>
                <p>{"Connecting..."}</p>
            </div>
        },

    };

    html! {
        <div class={ClassName::PAGE}>
            <style>{STYLE_SHEET}</style>
            { body }
        </div>
    }
}


enum RoomAction {
    Join(RoomId, GameKind),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct RoomState {
    room_id: Option<RoomId>,
    game_kind: Option<GameKind>,
}

impl Default for RoomState {
    fn default() -> Self {
        Self { room_id: None, game_kind: None, }
    }
}

impl Reducible for RoomState {
    type Action = RoomAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            RoomAction::Join(room_id, game_kind) => {
                Self { 
                    room_id: Some(room_id),
                    game_kind: Some(game_kind),
                    ..*self
                }.into()
            },
        }
    }
}

struct SocketHandler {
    dispatcher: UseReducerDispatcher<RoomState>,
}

impl SocketHandler {
    fn spawn(self, ws: WebSocket) -> SocketTearDown {
        let (send, recv) = futures::channel::oneshot::channel();
        spawn_local(async move {
            self.handle_socket(ws, recv).await;
        });
        SocketTearDown { send }
    }

    async fn handle_socket(self, ws: WebSocket, recv: Receiver<()>) {
        let (mut write, mut read) = ws.split();
        let recv = recv.into_future().shared();
        loop {
            let next_msg = select(read.next(), recv.clone()).await;
            match next_msg {
                Either::Left((Some(msg), _)) => {
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
                        self.handle_msg(msg).await;
                        // Self::handle_msg(msg, &self.dispatcher).await;
                    }
                }
                Either::Right((Ok(_), _)) => {
                    // Handle the recv stream event
                    break;
                }
                _ => break,
            }
        }

        log::info!("Socket closed");
    }

    async fn handle_msg(&self, msg: ServerMsg) -> Result<(), Box<dyn Error>> {
        log::info!("Server message: {:?}", msg);

        match msg {
            ServerMsg::RoomJoined(room_id, room) => {
                self.dispatcher.dispatch(RoomAction::Join(room_id, room.game));
            }
        }

        Ok(())
    }
}

struct SocketTearDown {
    send: Sender<()>,
}

impl TearDown for SocketTearDown {
    fn tear_down(self) {
        self.send.send(());
    }
}
