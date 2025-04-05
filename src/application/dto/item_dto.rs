use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ItemResponse {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
}