use axum::{Router, response::IntoResponse, routing::get, http::Response};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

const TOPICS_RON: &str = include_str!("gecko-topics.ron");

#[derive(Serialize, Deserialize, Debug)]
struct Topic {
    name: String,
    words: Vec<Vec<String>>, // TODO Figure out how to make this work as a [[String;4]; 4]
}

lazy_static! {
    static ref TOPICS: Vec<Topic> = ron::from_str(&TOPICS_RON).unwrap();
}

pub fn gecko_api() -> Router {
  Router::new()
    .route("/random-card", get(random_card))
}

async fn random_card() -> impl IntoResponse {
    let choice: usize = 0; // TODO actually make this random
    let topic = &TOPICS[choice];

    let serialized = ron::to_string(topic).unwrap();

    Response::builder()
        .body(serialized)
        .unwrap()
}