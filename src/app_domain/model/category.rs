use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryPath {
    pub path: Vec<String>,
    pub depth: usize,
}

impl CategoryPath {
    pub fn new(path: Vec<String>) -> Self {
        let depth = path.len();
        Self { path, depth }
    }

    // pub fn is_valid_depth(&self) -> bool {
    //     self.depth <= 5
    // }

    pub fn contains(&self, category_id: &str) -> bool {
        self.path.contains(&category_id.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryTree {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub children: Vec<CategoryTree>,
}

impl Category {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        parent_id: Option<String>,
        sort_order: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            parent_id,
            sort_order,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate_name(&self) -> Result<(), CategoryError> {
        if self.name.is_empty() {
            return Err(CategoryError::InvalidName("カテゴリ名は必須です".to_string()));
        }
        if self.name.len() > 100 {
            return Err(CategoryError::InvalidName("カテゴリ名は100文字以下である必要があります".to_string()));
        }
        Ok(())
    }

    pub fn validate_sort_order(&self) -> Result<(), CategoryError> {
        if self.sort_order < 0 {
            return Err(CategoryError::InvalidSortOrder("表示順序は0以上である必要があります".to_string()));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), CategoryError> {
        self.validate_name()?;
        self.validate_sort_order()?;
        Ok(())
    }

    pub fn update_name(&mut self, name: String) -> Result<(), CategoryError> {
        if name.is_empty() {
            return Err(CategoryError::InvalidName("カテゴリ名は必須です".to_string()));
        }
        if name.len() > 100 {
            return Err(CategoryError::InvalidName("カテゴリ名は100文字以下である必要があります".to_string()));
        }
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn update_sort_order(&mut self, sort_order: i32) -> Result<(), CategoryError> {
        if sort_order < 0 {
            return Err(CategoryError::InvalidSortOrder("表示順序は0以上である必要があります".to_string()));
        }
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryError {
    NotFound(String),
    InvalidName(String),
    InvalidSortOrder(String),
    NameDuplicate(String),
    CircularReference(String),
    MaxDepthExceeded(String),
    HasChildren(String),
    // HasProducts(String),
}

impl std::fmt::Display for CategoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CategoryError::NotFound(msg) => write!(f, "Category not found: {}", msg),
            CategoryError::InvalidName(msg) => write!(f, "Invalid category name: {}", msg),
            CategoryError::InvalidSortOrder(msg) => write!(f, "Invalid sort order: {}", msg),
            CategoryError::NameDuplicate(msg) => write!(f, "Category name duplicate: {}", msg),
            CategoryError::CircularReference(msg) => write!(f, "Circular reference detected: {}", msg),
            CategoryError::MaxDepthExceeded(msg) => write!(f, "Maximum depth exceeded: {}", msg),
            CategoryError::HasChildren(msg) => write!(f, "Category has children: {}", msg),
            // CategoryError::HasProducts(msg) => write!(f, "Category has products: {}", msg),
        }
    }
}

impl std::error::Error for CategoryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            Some("Electronic devices".to_string()),
            None,
            1,
        );

        assert_eq!(category.id, "cat_123");
        assert_eq!(category.name, "Electronics");
        assert_eq!(category.description, Some("Electronic devices".to_string()));
        assert_eq!(category.parent_id, None);
        assert_eq!(category.sort_order, 1);
        assert!(category.is_active);
    }

    #[test]
    fn test_category_validation_valid() {
        let category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            None,
            None,
            1,
        );

        assert!(category.validate().is_ok());
    }

    #[test]
    fn test_category_validation_empty_name() {
        let category = Category::new(
            "cat_123".to_string(),
            "".to_string(),
            None,
            None,
            1,
        );

        assert!(category.validate().is_err());
        match category.validate() {
            Err(CategoryError::InvalidName(msg)) => assert_eq!(msg, "カテゴリ名は必須です"),
            _ => panic!("Expected InvalidName error"),
        }
    }

    #[test]
    fn test_category_validation_long_name() {
        let long_name = "a".repeat(101);
        let category = Category::new(
            "cat_123".to_string(),
            long_name,
            None,
            None,
            1,
        );

        assert!(category.validate().is_err());
        match category.validate() {
            Err(CategoryError::InvalidName(msg)) => assert_eq!(msg, "カテゴリ名は100文字以下である必要があります"),
            _ => panic!("Expected InvalidName error"),
        }
    }

    #[test]
    fn test_category_validation_negative_sort_order() {
        let category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            None,
            None,
            -1,
        );

        assert!(category.validate().is_err());
        match category.validate() {
            Err(CategoryError::InvalidSortOrder(msg)) => assert_eq!(msg, "表示順序は0以上である必要があります"),
            _ => panic!("Expected InvalidSortOrder error"),
        }
    }

    #[test]
    fn test_category_update_name() {
        let mut category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            None,
            None,
            1,
        );

        let original_updated_at = category.updated_at;
        
        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        assert!(category.update_name("Updated Electronics".to_string()).is_ok());
        assert_eq!(category.name, "Updated Electronics");
        assert!(category.updated_at > original_updated_at);
    }

    #[test]
    fn test_category_update_name_invalid() {
        let mut category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            None,
            None,
            1,
        );

        assert!(category.update_name("".to_string()).is_err());
        assert_eq!(category.name, "Electronics"); // Should remain unchanged
    }

    #[test]
    fn test_category_deactivate() {
        let mut category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            None,
            None,
            1,
        );

        let original_updated_at = category.updated_at;
        
        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        category.deactivate();
        assert!(!category.is_active);
        assert!(category.updated_at > original_updated_at);
    }

    #[test]
    fn test_category_path_creation() {
        let path = CategoryPath::new(vec!["cat_1".to_string(), "cat_2".to_string(), "cat_3".to_string()]);
        
        assert_eq!(path.path.len(), 3);
        assert_eq!(path.depth, 3);
        // assert!(path.is_valid_depth());
        assert!(path.contains("cat_2"));
        assert!(!path.contains("cat_4"));
    }

    #[test]
    fn test_category_path_max_depth() {
        let path = CategoryPath::new(vec![
            "cat_1".to_string(),
            "cat_2".to_string(),
            "cat_3".to_string(),
            "cat_4".to_string(),
            "cat_5".to_string(),
        ]);
        
        assert_eq!(path.depth, 5);
        // assert!(path.is_valid_depth());
        
        let path_too_deep = CategoryPath::new(vec![
            "cat_1".to_string(),
            "cat_2".to_string(),
            "cat_3".to_string(),
            "cat_4".to_string(),
            "cat_5".to_string(),
            "cat_6".to_string(),
        ]);
        
        assert_eq!(path_too_deep.depth, 6);
        // assert!(!path_too_deep.is_valid_depth());
    }
}