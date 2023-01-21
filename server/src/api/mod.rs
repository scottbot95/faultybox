mod gecko;

use axum::{routing::get, Router};

use crate::hello;

use self::gecko::gecko_api;

pub fn api_router() -> Router {
    Router::new()
        .route("/hello", get(hello))
        .nest("/gecko", gecko_api())
}
