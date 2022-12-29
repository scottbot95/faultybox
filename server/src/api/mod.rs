mod gecko;

use axum::{Router, routing::get};

use crate::hello;

use self::gecko::gecko_api;

pub fn api_router() -> Router {
    Router::new()
        .route("/hello", get(hello))
        .nest("/gecko", gecko_api())
}
