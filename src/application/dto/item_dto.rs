use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ItemResponse {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct BatchDeleteRequest {
    pub ids: Vec<u64>,
    pub is_physical: Option<bool>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct BatchDeleteResponse {
    pub successful_ids: Vec<u64>,
    pub failed_ids: Vec<u64>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DeletionValidationResponse {
    pub can_delete: bool,
    pub related_orders: i64,
    pub related_reviews: i64,
    pub related_categories: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DeletionLogResponse {
    pub id: u64,
    pub item_id: u64,
    pub item_name: String,
    pub deletion_type: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_item_request() {
        let req = CreateItemRequest {
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
        };

        assert_eq!(req.name, "Test Item");
        assert_eq!(req.description, Some("Test Description".to_string()));

        let req_no_desc = CreateItemRequest {
            name: "No Description".to_string(),
            description: None,
        };

        assert_eq!(req_no_desc.name, "No Description");
        assert_eq!(req_no_desc.description, None);
    }

    #[test]
    fn test_update_item_request() {
        let req = UpdateItemRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated Description".to_string()),
        };

        assert_eq!(req.name, Some("Updated Name".to_string()));
        assert_eq!(req.description, Some("Updated Description".to_string()));

        let req_partial = UpdateItemRequest {
            name: Some("Only Name".to_string()),
            description: None,
        };

        assert_eq!(req_partial.name, Some("Only Name".to_string()));
        assert_eq!(req_partial.description, None);
    }

    #[test]
    fn test_item_response() {
        let resp = ItemResponse {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        assert_eq!(resp.id, 1);
        assert_eq!(resp.name, "Test Item");
        assert_eq!(resp.description, Some("Test Description".to_string()));
        assert!(!resp.deleted);
        assert_eq!(resp.deleted_at, None);
    }
}
