use crate::AppState;
use axum::{extract::Path, http::Response, response::IntoResponse, routing::get, Router};
use lazy_static::lazy_static;
use models::Topic;
use rand::{thread_rng, Rng};

const TOPICS_RON: &str = include_str!("gecko-topics.ron");

lazy_static! {
    static ref TOPICS: Vec<Topic> = ron::from_str(TOPICS_RON).unwrap();
}

pub fn gecko_api() -> Router<AppState> {
    Router::new()
        .route("/random-card", get(random_card))
        .route("/topics", get(list_topics))
        .route("/topics/{id}", get(get_topic))
}

async fn list_topics() -> impl IntoResponse {
    let topics = TOPICS.iter().map(|t| t.name.as_str()).collect::<Vec<_>>();

    let serialized = serde_json::to_string(&topics).unwrap();
    Response::builder().body(serialized).unwrap()
}

async fn get_topic(Path(topic_id): Path<usize>) -> impl IntoResponse {
    if let Some(topic) = TOPICS.get(topic_id) {
        let serialized = serde_json::to_string(topic).unwrap();
        Response::builder().body(serialized).unwrap()
    } else {
        Response::builder()
            .status(404)
            .body(format!("Unkonwn topic: {topic_id}"))
            .unwrap()
    }
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
