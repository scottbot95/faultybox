use std::fmt::format;
use std::rc::Rc;
use gloo_net::http::Request;
use gloo_net::websocket::futures::WebSocket;
use wasm_bindgen::{JsError, JsValue};
use web_sys::window;
use yew::{hook, use_context};
use models::GameKind;
use models::room::api::JoinRoomOutput;
use models::room::RoomId;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Eq, PartialEq)]
pub struct ApiClientContext<T: ApiClient>(pub Rc<T>);

#[hook]
pub fn use_api<T: ApiClient + Clone + PartialEq + 'static>() -> Rc<T> {
    let ctx = use_context::<ApiClientContext<T>>();
    ctx.unwrap().0
}

pub trait ApiClient {
    async fn create_room(&self, game: GameKind) -> Result<JoinRoomOutput>;
    async fn join_room(&self, room_id: &RoomId) -> Result<JoinRoomOutput>;

    fn connect_room(&self) -> Result<WebSocket>;
}

#[derive(Clone, PartialEq)]
pub struct ApiClientImpl {
    host: String,
}

impl ApiClientImpl {
    pub fn new_from_window() -> std::result::Result<Self, JsValue> {
        let host = window().map(|w| w.location().host())
            .unwrap_or_else(|| {
                log::warn!("No window object available");
                Ok("localhost:8080".to_string())
            })?;

        Ok(Self { host })
    }
}

impl ApiClient for ApiClientImpl {
    async fn create_room(&self, game: GameKind) -> Result<JoinRoomOutput>{
        let resp = Request::post(&format!("/api/room/create/{}", game))
            .send()
            .await?;

        let result = resp.json().await?;
        Ok(result)
    }

    async fn join_room(&self, room_id: &RoomId) -> Result<JoinRoomOutput> {
        let resp = Request::post(&format!("/api/room/join/{}", room_id))
            .send()
            .await?;

        let result = resp.json().await?;
        Ok(result)
    }

    fn connect_room(&self) -> Result<WebSocket> {
        let url = format!("ws://{}/api/room/connect", self.host);
        let ws = WebSocket::open(&url)?;
        Ok(ws)
    }
}
