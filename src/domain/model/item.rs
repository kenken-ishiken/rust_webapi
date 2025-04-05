use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
}