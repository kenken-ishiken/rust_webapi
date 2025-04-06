use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_user_equality() {
        let user1 = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let user2 = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let user3 = User {
            id: 2,
            username: "otheruser".to_string(),
            email: "other@example.com".to_string(),
        };

        assert_eq!(user1, user2);
        assert_ne!(user1, user3);
    }
}
