pub mod room;
pub mod ws;

use serde::{Deserialize, Serialize};

pub const TOPIC_GRID_ROWS: usize = 4;
pub const TOPIC_GRID_COLS: usize = 4;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    // #[serde(with = "serde_arrays")]
    pub words: Vec<String>,
}
