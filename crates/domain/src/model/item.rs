use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DeletionType {
    Logical,
    Physical,
    Restore,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RelatedDataCount {
    pub related_orders: i64,
    pub related_reviews: i64,
    pub related_categories: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeletionValidation {
    pub can_delete: bool,
    pub related_data: RelatedDataCount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeletionLog {
    pub id: u64,
    pub item_id: u64,
    pub item_name: String,
    pub deletion_type: DeletionType,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_creation() {
        let item = Item {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, Some("Test Description".to_string()));
        assert_eq!(item.deleted, false);
        assert_eq!(item.deleted_at, None);
    }

    #[test]
    fn test_item_without_description() {
        let item = Item {
            id: 2,
            name: "No Description Item".to_string(),
            description: None,
            deleted: false,
            deleted_at: None,
        };

        assert_eq!(item.id, 2);
        assert_eq!(item.name, "No Description Item");
        assert_eq!(item.description, None);
        assert_eq!(item.deleted, false);
        assert_eq!(item.deleted_at, None);
    }
}
