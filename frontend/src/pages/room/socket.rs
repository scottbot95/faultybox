use std::pin::pin;
use futures::channel::oneshot::{Receiver, Sender};
use futures::{select, StreamExt, FutureExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use yew::platform::spawn_local;
use yew::{TearDown, UseReducerDispatcher};
use models::ws::ServerMsg;
use crate::pages::room::reducer::RoomState;

pub(super) type Dispatcher = UseReducerDispatcher<RoomState>;

pub(super) struct SocketHandler {
    pub(super) dispatcher: Dispatcher,
}

impl SocketHandler {
    pub(super) fn spawn(self, ws: WebSocket) -> SocketTearDown {
        let (control_tx, recv) = futures::channel::oneshot::channel();
        spawn_local(async move {
            self.handle_socket(ws, recv).await;
        });
        SocketTearDown::CloseSocket { control_tx }
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
                self.dispatcher.dispatch(super::reducer::RoomAction::UpdateRoom(room));
            }
        }
    }
}

pub(super) enum SocketTearDown {
    Nop,
    CloseSocket {
        control_tx: Sender<()>
    },
}

impl TearDown for SocketTearDown {
    fn tear_down(self) {
        match self {
            SocketTearDown::Nop => {}
            SocketTearDown::CloseSocket {control_tx} => {
                if control_tx.send(()).is_err() {
                    log::warn!("Websocket closed already");
                }
            }
        }
    }
}