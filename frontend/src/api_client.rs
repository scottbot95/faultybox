use gloo_net::http::Request;
use gloo_net::websocket::futures::WebSocket;
use models::room::api::{CreateRoomInput, JoinRoomInput, JoinRoomOutput};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::window;
use yew::{hook, use_context};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Eq, PartialEq)]
pub struct ApiClientContext<T: ApiClient>(pub Rc<T>);

#[hook]
pub fn use_api<T>() -> Rc<T> 
where
    T: ApiClient + Clone + PartialEq + 'static
{
    let ctx = use_context::<ApiClientContext<T>>();
    ctx.unwrap().0
}

pub trait ApiClient {
    async fn create_room(&self, input: CreateRoomInput) -> Result<JoinRoomOutput>;
    async fn join_room(&self, input: JoinRoomInput) -> Result<JoinRoomOutput>;

    fn connect_room(&self) -> Result<WebSocket>;
}

#[derive(Clone, PartialEq)]
pub struct ApiClientImpl {
    host: String,
}

impl ApiClientImpl {
    pub fn new_from_window() -> std::result::Result<Self, JsValue> {
        let host = window().map(|w| w.location().host()).unwrap_or_else(|| {
            log::warn!("No window object available");
            Ok("localhost:8080".to_string())
        })?;

        Ok(Self { host })
    }

    fn build_request<B: Serialize>(&self, url: &str, body: &B) -> Result<Request> {
        let req = Request::post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(body)?)?;

        Ok(req)
    }

    async fn do_request<B: Serialize, O: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<O> {
        let resp = self.build_request(url, body)?.send().await?;

        let result = resp.json().await?;
        Ok(result)
    }
}

impl ApiClient for ApiClientImpl {
    async fn create_room(&self, input: CreateRoomInput) -> Result<JoinRoomOutput> {
        self.do_request("/api/room/create", &input).await
    }

    async fn join_room(&self, input: JoinRoomInput) -> Result<JoinRoomOutput> {
        self.do_request("/api/room/join", &input).await
    }

    fn connect_room(&self) -> Result<WebSocket> {
        let url = format!("ws://{}/api/room/connect", self.host);
        let ws = WebSocket::open(&url)?;
        Ok(ws)
    }
}
