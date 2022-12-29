use axum::{http::Response, response::IntoResponse, routing::get, Router};
use lazy_static::lazy_static;
use models::Topic;
use rand::{thread_rng, Rng};

const TOPICS_RON: &str = include_str!("gecko-topics.ron");

lazy_static! {
    static ref TOPICS: Vec<Topic> = ron::from_str(TOPICS_RON).unwrap();
}

pub fn gecko_api() -> Router {
    Router::new().route("/random-card", get(random_card))
}

async fn random_card() -> impl IntoResponse {
    let mut rng = thread_rng();
    let topic = choose_random_card(&mut rng);

    let serialized = serde_json::to_string(topic).unwrap();

    Response::builder().body(serialized).unwrap()
}

fn choose_random_card<R: Rng + ?Sized>(rng: &mut R) -> &Topic {
    let choice = rng.gen_range(0..TOPICS.len());
    &TOPICS[choice]
}
