use crate::domain::model::user::User;
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