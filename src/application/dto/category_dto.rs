use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    pub parent_id: Option<String>,
    pub sort_order: i32,
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
    pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct CategoryErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<CategoryErrorDetails>,
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
            path: path.path.into_iter().map(|id| CategoryPathItem {
                id: id.clone(),
                name: id, // This would need to be enriched with actual names from the repository
            }).collect(),
            depth: path.depth,
        }
    }
}

impl From<crate::app_domain::model::category::CategoryError> for CategoryErrorResponse {
    fn from(error: crate::app_domain::model::category::CategoryError) -> Self {
        use crate::app_domain::model::category::CategoryError;
        
        match error {
            CategoryError::NotFound(msg) => Self {
                code: "CATEGORY_NOT_FOUND".to_string(),
                message: msg,
                details: None,
            },
            CategoryError::InvalidName(msg) => Self {
                code: "CATEGORY_INVALID_NAME".to_string(),
                message: msg,
                details: Some(CategoryErrorDetails {
                    field: Some("name".to_string()),
                    value: None,
                    parent_id: None,
                }),
            },
            CategoryError::InvalidSortOrder(msg) => Self {
                code: "CATEGORY_INVALID_SORT_ORDER".to_string(),
                message: msg,
                details: Some(CategoryErrorDetails {
                    field: Some("sort_order".to_string()),
                    value: None,
                    parent_id: None,
                }),
            },
            CategoryError::NameDuplicate(msg) => Self {
                code: "CATEGORY_NAME_DUPLICATE".to_string(),
                message: msg,
                details: Some(CategoryErrorDetails {
                    field: Some("name".to_string()),
                    value: None,
                    parent_id: None,
                }),
            },
            CategoryError::CircularReference(msg) => Self {
                code: "CATEGORY_CIRCULAR_REFERENCE".to_string(),
                message: msg,
                details: None,
            },
            CategoryError::MaxDepthExceeded(msg) => Self {
                code: "CATEGORY_MAX_DEPTH_EXCEEDED".to_string(),
                message: msg,
                details: None,
            },
            CategoryError::HasChildren(msg) => Self {
                code: "CATEGORY_HAS_CHILDREN".to_string(),
                message: msg,
                details: None,
            },
            CategoryError::HasProducts(msg) => Self {
                code: "CATEGORY_HAS_PRODUCTS".to_string(),
                message: msg,
                details: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_domain::model::category::{Category, CategoryError};

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
        let error = CategoryError::NotFound("Category not found".to_string());
        let response: CategoryErrorResponse = error.into();

        assert_eq!(response.code, "CATEGORY_NOT_FOUND");
        assert_eq!(response.message, "Category not found");
        assert!(response.details.is_none());
    }

    #[test]
    fn test_category_error_with_details_conversion() {
        let error = CategoryError::InvalidName("Name is required".to_string());
        let response: CategoryErrorResponse = error.into();

        assert_eq!(response.code, "CATEGORY_INVALID_NAME");
        assert_eq!(response.message, "Name is required");
        assert!(response.details.is_some());
        
        let details = response.details.unwrap();
        assert_eq!(details.field, Some("name".to_string()));
    }

    #[test]
    fn test_create_category_request_deserialization() {
        let json = r#"
        {
            "name": "Electronics",
            "description": "Electronic devices",
            "parent_id": null,
            "sort_order": 1
        }
        "#;

        let request: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Electronics");
        assert_eq!(request.description, Some("Electronic devices".to_string()));
        assert_eq!(request.parent_id, None);
        assert_eq!(request.sort_order, 1);
    }

    #[test]
    fn test_update_category_request_deserialization() {
        let json = r#"
        {
            "name": "Updated Electronics",
            "sort_order": 2
        }
        "#;

        let request: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated Electronics".to_string()));
        assert_eq!(request.description, None);
        assert_eq!(request.sort_order, Some(2));
        assert_eq!(request.is_active, None);
    }

    #[test]
    fn test_move_category_request_deserialization() {
        let json = r#"
        {
            "parent_id": "cat_456",
            "sort_order": 3
        }
        "#;

        let request: MoveCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.parent_id, Some("cat_456".to_string()));
        assert_eq!(request.sort_order, 3);
    }

    #[test]
    fn test_category_query_params_deserialization() {
        let json = r#"
        {
            "parent_id": "cat_123",
            "include_inactive": true,
            "sort": "name"
        }
        "#;

        let params: CategoryQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.parent_id, Some("cat_123".to_string()));
        assert_eq!(params.include_inactive, Some(true));
        assert_eq!(params.sort, Some("name".to_string()));
    }
}