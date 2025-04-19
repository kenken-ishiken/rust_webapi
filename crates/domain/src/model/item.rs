use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
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
        };

        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, Some("Test Description".to_string()));
    }

    #[test]
    fn test_item_without_description() {
        let item = Item {
            id: 2,
            name: "No Description Item".to_string(),
            description: None,
        };

        assert_eq!(item.id, 2);
        assert_eq!(item.name, "No Description Item");
        assert_eq!(item.description, None);
    }
}
