use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::application::service::user_service::UserService;

// Include the generated proto code
tonic::include_proto!("user");

pub use user_service_server::{UserService as UserServiceTrait, UserServiceServer};

#[derive(Clone)]
pub struct UserServiceImpl {
    service: Arc<UserService>,
}

impl UserServiceImpl {
    pub fn new(service: Arc<UserService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl UserServiceTrait for UserServiceImpl {
    async fn get_users(
        &self,
        _request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        info!("gRPC: Getting all users");

        match self.service.find_all().await {
            Ok(users) => {
                info!("gRPC: Fetched {} users", users.len());

                let users_proto = users
                    .into_iter()
                    .map(|user| User {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                    })
                    .collect();

                let response = GetUsersResponse { users: users_proto };
                Ok(Response::new(response))
            }
            Err(err) => {
                error!("gRPC: Failed to fetch users: {}", err);
                Err(Status::internal("Failed to fetch users"))
            }
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC: Getting user with id {}", req.id);

        match self.service.find_by_id(req.id).await {
            Ok(user) => {
                info!("gRPC: Found user {}", req.id);

                let user_proto = Some(User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                });

                let response = GetUserResponse { user: user_proto };
                Ok(Response::new(response))
            }
            Err(err) => match err {
                crate::infrastructure::error::AppError::NotFound(_) => {
                    info!("gRPC: User {} not found", req.id);
                    Err(Status::not_found("User not found"))
                }
                _ => {
                    error!("gRPC: Failed to get user {}: {}", req.id, err);
                    Err(Status::internal("Failed to get user"))
                }
            },
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC: Creating user with username {}", req.username);

        let create_request = crate::application::dto::user_dto::CreateUserRequest {
            username: req.username,
            email: req.email,
        };

        match self.service.create(create_request).await {
            Ok(new_user) => {
                info!("gRPC: Created user with id {}", new_user.id);

                let user_proto = Some(User {
                    id: new_user.id,
                    username: new_user.username,
                    email: new_user.email,
                });

                let response = CreateUserResponse { user: user_proto };
                Ok(Response::new(response))
            }
            Err(err) => {
                error!("gRPC: Failed to create user: {}", err);
                Err(Status::internal("Failed to create user"))
            }
        }
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC: Updating user {}", req.id);

        // gRPCのUpdateUserRequestはOption<String>フィールドを持つ
        let update_request = crate::application::dto::user_dto::UpdateUserRequest {
            username: req.username.filter(|s| !s.is_empty()),
            email: req.email.filter(|s| !s.is_empty()),
        };

        match self.service.update(req.id, update_request).await {
            Ok(updated_user) => {
                info!("gRPC: Updated user {}", req.id);

                let user_proto = Some(User {
                    id: updated_user.id,
                    username: updated_user.username,
                    email: updated_user.email,
                });

                let response = UpdateUserResponse { user: user_proto };
                Ok(Response::new(response))
            }
            Err(err) => match err {
                crate::infrastructure::error::AppError::NotFound(_) => {
                    info!("gRPC: User {} not found for update", req.id);
                    Err(Status::not_found("User not found"))
                }
                _ => {
                    error!("gRPC: Failed to update user {}: {}", req.id, err);
                    Err(Status::internal("Failed to update user"))
                }
            },
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC: Deleting user {}", req.id);

        match self.service.delete(req.id).await {
            Ok(success) => {
                info!("gRPC: Delete user {} result: {}", req.id, success);
                let response = DeleteUserResponse { success };
                Ok(Response::new(response))
            }
            Err(err) => match err {
                crate::infrastructure::error::AppError::NotFound(_) => {
                    info!("gRPC: User {} not found for deletion", req.id);
                    Err(Status::not_found("User not found"))
                }
                _ => {
                    error!("gRPC: Failed to delete user {}: {}", req.id, err);
                    Err(Status::internal("Failed to delete user"))
                }
            },
        }
    }
}
