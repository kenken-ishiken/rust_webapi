use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct MoveCategoryRequest {
    pub new_parent_id: Option<String>,
    pub new_sort_order: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct CategoryListResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub children_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct CategoryTreeResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub children: Vec<CategoryTreeResponse>,
}

#[derive(Serialize)]
pub struct CategoryPathItem {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct CategoryPathResponse {
    pub path: Vec<CategoryPathItem>,
    pub depth: usize,
}

#[derive(Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryListResponse>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct CategoryTreesResponse {
    pub tree: Vec<CategoryTreeResponse>,
}

#[derive(Deserialize)]
pub struct CategoryQueryParams {
    pub parent_id: Option<String>,
    pub include_inactive: Option<bool>,
    // pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct CategoryErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct CategoryErrorDetails {
    pub field: Option<String>,
    pub value: Option<String>,
    pub parent_id: Option<String>,
}

impl From<crate::app_domain::model::category::Category> for CategoryResponse {
    fn from(category: crate::app_domain::model::category::Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            description: category.description,
            parent_id: category.parent_id,
            sort_order: category.sort_order,
            is_active: category.is_active,
            created_at: category.created_at,
            updated_at: category.updated_at,
        }
    }
}

impl From<crate::app_domain::model::category::CategoryTree> for CategoryTreeResponse {
    fn from(tree: crate::app_domain::model::category::CategoryTree) -> Self {
        Self {
            id: tree.id,
            name: tree.name,
            description: tree.description,
            sort_order: tree.sort_order,
            is_active: tree.is_active,
            children: tree.children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<crate::app_domain::model::category::CategoryPath> for CategoryPathResponse {
    fn from(path: crate::app_domain::model::category::CategoryPath) -> Self {
        Self {
            path: path
                .path
                .into_iter()
                .map(|id| CategoryPathItem {
                    id: id.clone(),
                    name: id, // This would need to be enriched with actual names from the repository
                })
                .collect(),
            depth: path.depth,
        }
    }
}

impl From<crate::app_domain::model::category::CategoryError> for CategoryErrorResponse {
    fn from(error: crate::app_domain::model::category::CategoryError) -> Self {
        use crate::app_domain::model::category::CategoryError;

        match &error {
            CategoryError::NotFound(_) => Self {
                code: "CATEGORY_NOT_FOUND".to_string(),
                message: error.to_string(),
                details: None,
            },
            CategoryError::InvalidName(_) => Self {
                code: "CATEGORY_INVALID_NAME".to_string(),
                message: error.to_string(),
                details: Some(serde_json::json!({
                    "field": "name",
                    "value": null,
                    "parent_id": null,
                })),
            },
            CategoryError::InvalidSortOrder(_) => Self {
                code: "CATEGORY_INVALID_SORT_ORDER".to_string(),
                message: error.to_string(),
                details: Some(serde_json::json!({
                    "field": "sort_order",
                    "value": null,
                    "parent_id": null,
                })),
            },
            CategoryError::NameDuplicate(_) => Self {
                code: "CATEGORY_NAME_DUPLICATE".to_string(),
                message: error.to_string(),
                details: Some(serde_json::json!({
                    "field": "name",
                    "value": null,
                    "parent_id": null,
                })),
            },
            CategoryError::CircularReference(_) => Self {
                code: "CATEGORY_CIRCULAR_REFERENCE".to_string(),
                message: error.to_string(),
                details: None,
            },
            CategoryError::MaxDepthExceeded(_) => Self {
                code: "CATEGORY_MAX_DEPTH_EXCEEDED".to_string(),
                message: error.to_string(),
                details: None,
            },
            CategoryError::HasChildren(_) => Self {
                code: "CATEGORY_HAS_CHILDREN".to_string(),
                message: error.to_string(),
                details: None,
            },
            // CategoryError::HasProducts(_) => Self {
            //     code: "CATEGORY_HAS_PRODUCTS".to_string(),
            //     message: error.to_string(),
            //     details: None,
            // },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_domain::model::category::{Category, CategoryError};
    use serde_json;

    #[test]
    fn test_category_response_conversion() {
        let category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            Some("Electronic devices".to_string()),
            None,
            1,
        );

        let response: CategoryResponse = category.clone().into();

        assert_eq!(response.id, category.id);
        assert_eq!(response.name, category.name);
        assert_eq!(response.description, category.description);
        assert_eq!(response.parent_id, category.parent_id);
        assert_eq!(response.sort_order, category.sort_order);
        assert_eq!(response.is_active, category.is_active);
    }

    #[test]
    fn test_category_error_response_conversion() {
        let error = CategoryError::NotFound("cat_123".to_string());
        let response: CategoryErrorResponse = error.into();

        assert_eq!(response.code, "CATEGORY_NOT_FOUND");
        assert_eq!(response.message, "Category not found: cat_123");
        assert!(response.details.is_none());
    }

    #[test]
    fn test_category_error_response_with_details() {
        let response = CategoryErrorResponse {
            code: "CATEGORY_ERROR".to_string(),
            message: "An error occurred".to_string(),
            details: Some(serde_json::json!({
                "field": "name",
                "reason": "too long"
            })),
        };

        let details = response.details.expect("Details should exist");
        assert_eq!(details["field"], "name");
        assert_eq!(details["reason"], "too long");
    }

    #[test]
    fn test_create_category_request_deserialization() {
        let json = r#"{
            "name": "Electronics",
            "description": "Electronic products",
            "parent_id": "cat_123",
            "sort_order": 1
        }"#;

        let request: CreateCategoryRequest =
            serde_json::from_str(json).expect("Failed to deserialize CreateCategoryRequest");

        assert_eq!(request.name, "Electronics");
        assert_eq!(request.description, Some("Electronic products".to_string()));
        assert_eq!(request.parent_id, Some("cat_123".to_string()));
        assert_eq!(request.sort_order, 1);
    }

    #[test]
    fn test_update_category_request_deserialization() {
        let json = r#"{
            "name": "Updated Electronics",
            "description": "Updated description",
            "sort_order": 2,
            "is_active": false
        }"#;

        let request: UpdateCategoryRequest =
            serde_json::from_str(json).expect("Failed to deserialize UpdateCategoryRequest");

        assert_eq!(request.name, Some("Updated Electronics".to_string()));
        assert_eq!(request.description, Some("Updated description".to_string()));
        assert_eq!(request.sort_order, Some(2));
        assert_eq!(request.is_active, Some(false));
    }

    #[test]
    fn test_move_category_request_deserialization() {
        let json = r#"{
            "new_parent_id": "cat_456",
            "new_sort_order": 3
        }"#;

        let request: MoveCategoryRequest =
            serde_json::from_str(json).expect("Failed to deserialize MoveCategoryRequest");

        assert_eq!(request.new_parent_id, Some("cat_456".to_string()));
        assert_eq!(request.new_sort_order, Some(3));
    }

    #[test]
    fn test_category_query_params_deserialization() {
        let json = r#"{
            "parent_id": "cat_789",
            "include_inactive": true
        }"#;

        let params: CategoryQueryParams =
            serde_json::from_str(json).expect("Failed to deserialize CategoryQueryParams");

        assert_eq!(params.parent_id, Some("cat_789".to_string()));
        assert_eq!(params.include_inactive, Some(true));
    }
}
