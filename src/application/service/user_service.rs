use crate::application::dto::user_dto::{CreateUserRequest, UpdateUserRequest};
use crate::infrastructure::metrics::Metrics;
use domain::model::user::User;

pub struct UserService {
    repository: domain::repository::user_repository::UserRepositoryImpl,
}

impl UserService {
    pub fn new(repository: domain::repository::user_repository::UserRepositoryImpl) -> Self {
        Self { repository }
    }

    pub async fn find_all(&self) -> Vec<User> {
        Metrics::with_timer("user", "find_all", async {
            let users = self.repository.find_all().await;
            Metrics::record_success("user", "find_all");
            users
        }).await
    }

    pub async fn find_by_id(&self, id: u64) -> Option<User> {
        Metrics::with_metrics("user", "find_by_id", async {
            let user = self.repository.find_by_id(id).await;
            match user {
                Some(user) => Ok(user),
                None => Err("User not found"),
            }
        }).await.ok()
    }

    pub async fn create(&self, req: CreateUserRequest) -> User {
        Metrics::with_timer("user", "create", async {
            // IDはリポジトリ/DB側で生成
            let user = User {
                id: 0,
                username: req.username,
                email: req.email,
            };
            let created_user = self.repository.create(user).await;
            Metrics::record_success("user", "create");
            created_user
        }).await
    }

    pub async fn update(&self, id: u64, req: UpdateUserRequest) -> Option<User> {
        Metrics::with_metrics("user", "update", async {
            if let Some(mut user) = self.repository.find_by_id(id).await {
                if let Some(username) = req.username {
                    user.username = username;
                }
                if let Some(email) = req.email {
                    user.email = email;
                }
                let updated_user = self.repository.update(user).await;
                Ok(updated_user)
            } else {
                Err("User not found")
            }
        }).await.ok().flatten()
    }

    pub async fn delete(&self, id: u64) -> bool {
        Metrics::with_metrics("user", "delete", async {
            let result = self.repository.delete(id).await;
            if result {
                Ok(result)
            } else {
                Err("Failed to delete user")
            }
        }).await.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::user_repository::UserRepository;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;

    mock! {
        UserRep {}
        #[async_trait::async_trait]
        impl UserRepository for UserRep {
            async fn find_all(&self) -> Vec<User>;
            async fn find_by_id(&self, id: u64) -> Option<User>;
            async fn create(&self, user: User) -> User;
            async fn update(&self, user: User) -> Option<User>;
            async fn delete(&self, id: u64) -> bool;
        }
    }

    #[tokio::test]
    async fn test_find_all() {
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_find_all().return_once(|| {
            vec![
                User {
                    id: 1,
                    username: "user1".to_string(),
                    email: "user1@example.com".to_string(),
                },
                User {
                    id: 2,
                    username: "user2".to_string(),
                    email: "user2@example.com".to_string(),
                },
            ]
        });

        let service = UserService::new(Arc::new(mock_repo));
        let result = service.find_all().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].username, "user1");
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].username, "user2");
    }

    #[tokio::test]
    async fn test_find_by_id_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| {
                Some(User {
                    id: 1,
                    username: "user1".to_string(),
                    email: "user1@example.com".to_string(),
                })
            });

        let service = UserService::new(Arc::new(mock_repo));
        let result = service.find_by_id(1).await;

        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "user1");
        assert_eq!(user.email, "user1@example.com");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let service = UserService::new(Arc::new(mock_repo));
        let result = service.find_by_id(999).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create() {
        let mut mock_repo = MockUserRep::new();
        mock_repo
            .expect_create()
            .withf(|user| user.username == "newuser" && user.email == "newuser@example.com")
            .return_once(|user| User {
                id: 42,
                username: user.username,
                email: user.email,
            });

        let service = UserService::new(Arc::new(mock_repo));
        let request = CreateUserRequest {
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
        };

        let result = service.create(request).await;

        assert_eq!(result.id, 42);
        assert_eq!(result.username, "newuser");
        assert_eq!(result.email, "newuser@example.com");
    }

    #[tokio::test]
    async fn test_update_found() {
        let mut mock_repo = MockUserRep::new();

        // First expect find_by_id
        mock_repo
            .expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| {
                Some(User {
                    id: 1,
                    username: "oldname".to_string(),
                    email: "old@example.com".to_string(),
                })
            });

        // Then expect update
        mock_repo
            .expect_update()
            .withf(|user| {
                user.id == 1 && user.username == "newname" && user.email == "new@example.com"
            })
            .return_once(Some);

        let service = UserService::new(Arc::new(mock_repo));
        let request = UpdateUserRequest {
            username: Some("newname".to_string()),
            email: Some("new@example.com".to_string()),
        };

        let result = service.update(1, request).await;

        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "newname");
        assert_eq!(user.email, "new@example.com");
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let service = UserService::new(Arc::new(mock_repo));
        let request = UpdateUserRequest {
            username: Some("newname".to_string()),
            email: Some("new@example.com".to_string()),
        };

        let result = service.update(999, request).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockUserRep::new();
        mock_repo
            .expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = UserService::new(Arc::new(mock_repo));
        let result = service.delete(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo
            .expect_delete()
            .with(eq(999u64))
            .return_once(|_| false);

        let service = UserService::new(Arc::new(mock_repo));
        let result = service.delete(999).await;

        assert!(!result);
    }
}
