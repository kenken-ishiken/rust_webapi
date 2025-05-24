use crate::model::user::User;
use std::sync::Arc;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Vec<User>;
    async fn find_by_id(&self, id: u64) -> Option<User>;
    async fn create(&self, user: User) -> User;
    async fn update(&self, user: User) -> Option<User>;
    async fn delete(&self, id: u64) -> bool;
}

pub type UserRepositoryImpl = Arc<dyn UserRepository>;

#[cfg(test)]
pub use mockall::predicate::*;
#[cfg(test)]
pub use mockall::mock;

#[cfg(test)]
mock! {
    pub UserRepo {}
    #[async_trait]
    impl UserRepository for UserRepo {
        async fn find_all(&self) -> Vec<User>;
        async fn find_by_id(&self, id: u64) -> Option<User>;
        async fn create(&self, user: User) -> User;
        async fn update(&self, user: User) -> Option<User>;
        async fn delete(&self, id: u64) -> bool;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_all_success() {
        let user1 = User {
            id: 1,
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
        };
        let user2 = User {
            id: 2,
            username: "user2".to_string(),
            email: "user2@example.com".to_string(),
        };

        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_find_all()
            .return_once(move || vec![user1.clone(), user2.clone()]);

        let result = mock_repo.find_all().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].username, "user1");
        assert_eq!(result[0].email, "user1@example.com");
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].username, "user2");
        assert_eq!(result[1].email, "user2@example.com");
    }

    #[tokio::test]
    async fn test_find_all_empty() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_find_all()
            .return_once(|| vec![]);

        let result = mock_repo.find_all().await;

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_find_by_id_found() {
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(user.clone()));

        let result = mock_repo.find_by_id(1).await;

        assert!(result.is_some());
        let found_user = result.unwrap();
        assert_eq!(found_user.id, 1);
        assert_eq!(found_user.username, "testuser");
        assert_eq!(found_user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let result = mock_repo.find_by_id(999).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_success() {
        let user = User {
            id: 1,
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
        };

        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_create()
            .with(function(move |u: &User| {
                u.id == 1 && u.username == "newuser" && u.email == "new@example.com"
            }))
            .return_once(move |user| user);

        let result = mock_repo.create(user.clone()).await;

        assert_eq!(result.id, 1);
        assert_eq!(result.username, "newuser");
        assert_eq!(result.email, "new@example.com");
    }

    #[tokio::test]
    async fn test_update_success() {
        let user = User {
            id: 1,
            username: "updateduser".to_string(),
            email: "updated@example.com".to_string(),
        };

        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_update()
            .with(function(move |u: &User| {
                u.id == 1 && u.username == "updateduser"
            }))
            .return_once(move |user| Some(user));

        let result = mock_repo.update(user.clone()).await;

        assert!(result.is_some());
        let updated = result.unwrap();
        assert_eq!(updated.id, 1);
        assert_eq!(updated.username, "updateduser");
        assert_eq!(updated.email, "updated@example.com");
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let user = User {
            id: 999,
            username: "nonexistent".to_string(),
            email: "none@example.com".to_string(),
        };

        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_update()
            .with(function(move |u: &User| u.id == 999))
            .return_once(|_| None);

        let result = mock_repo.update(user).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let result = mock_repo.delete(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo.expect_delete()
            .with(eq(999u64))
            .return_once(|_| false);

        let result = mock_repo.delete(999).await;

        assert!(!result);
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let user = User {
            id: 1,
            username: "multiop".to_string(),
            email: "multi@example.com".to_string(),
        };

        let user_clone = user.clone();
        
        let mut mock_repo = MockUserRepo::new();
        
        // Expect create operation
        mock_repo.expect_create()
            .with(function(move |u: &User| u.id == 1))
            .return_once(move |user| user);
            
        // Expect find operation
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(user_clone.clone()));

        // Perform operations
        let created = mock_repo.create(user.clone()).await;
        assert_eq!(created.id, 1);
        
        let found = mock_repo.find_by_id(1).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
    }
}