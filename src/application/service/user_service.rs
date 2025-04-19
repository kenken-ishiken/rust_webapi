use domain::model::user::User;
use crate::application::dto::user_dto::{CreateUserRequest, UpdateUserRequest};
use std::sync::Mutex;
use crate::infrastructure::metrics::{increment_success_counter, increment_error_counter};

pub struct UserService {
    repository: domain::repository::user_repository::UserRepositoryImpl,
    counter: Mutex<u64>,
}

impl UserService {
    pub fn new(repository: domain::repository::user_repository::UserRepositoryImpl) -> Self {
        Self {
            repository,
            counter: Mutex::new(0),
        }
    }

    pub async fn find_all(&self) -> Vec<User> {
        let users = self.repository.find_all().await;
        increment_success_counter("user", "find_all");
        users
    }

    pub async fn find_by_id(&self, id: u64) -> Option<User> {
        let user = self.repository.find_by_id(id).await;
        if user.is_some() {
            increment_success_counter("user", "find_by_id");
        } else {
            increment_error_counter("user", "find_by_id");
        }
        user
    }

    pub async fn create(&self, req: CreateUserRequest) -> User {
        let mut counter = self.counter.lock().unwrap();
        let id = *counter;
        *counter += 1;

        let user = User {
            id,
            username: req.username,
            email: req.email,
        };

        let created_user = self.repository.create(user).await;
        increment_success_counter("user", "create");
        created_user
    }

    pub async fn update(&self, id: u64, req: UpdateUserRequest) -> Option<User> {
        if let Some(mut user) = self.repository.find_by_id(id).await {
            if let Some(username) = req.username {
                user.username = username;
            }
            if let Some(email) = req.email {
                user.email = email;
            }
            self.repository.update(user).await
        } else {
            increment_error_counter("user", "update");
            None
        }
    }

    pub async fn delete(&self, id: u64) -> bool {
        let result = self.repository.delete(id).await;
        if result {
            increment_success_counter("user", "delete");
        } else {
            increment_error_counter("user", "delete");
        }
        result
    }
}
