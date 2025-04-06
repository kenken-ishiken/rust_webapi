use crate::domain::model::user::User;
use crate::application::dto::user_dto::{CreateUserRequest, UpdateUserRequest};
use std::sync::Mutex;

pub struct UserService {
    repository: crate::domain::repository::user_repository::UserRepositoryImpl,
    counter: Mutex<u64>,
}

impl UserService {
    pub fn new(repository: crate::domain::repository::user_repository::UserRepositoryImpl) -> Self {
        Self {
            repository,
            counter: Mutex::new(0),
        }
    }

    pub async fn find_all(&self) -> Vec<User> {
        self.repository.find_all().await
    }

    pub async fn find_by_id(&self, id: u64) -> Option<User> {
        self.repository.find_by_id(id).await
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

        self.repository.create(user).await
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
            None
        }
    }

    pub async fn delete(&self, id: u64) -> bool {
        self.repository.delete(id).await
    }
}