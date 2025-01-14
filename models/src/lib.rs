pub mod room;
pub mod ws;

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

pub const TOPIC_GRID_ROWS: usize = 4;
pub const TOPIC_GRID_COLS: usize = 4;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    // #[serde(with = "serde_arrays")]
    pub words: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub enum GameKind {
    Gecko,
}

impl GameKind {
    pub fn values() -> &'static [GameKind] {
        &[GameKind::Gecko]
    }
}

impl Display for GameKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameKind::Gecko => write!(f, "Gecko"),
        }
    }
}