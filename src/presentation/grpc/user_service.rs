use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

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
        let users = self.service.find_all().await;
        info!("gRPC: Fetched {} users", users.len());

        let grpc_users = users
            .into_iter()
            .map(|user| User {
                id: user.id,
                username: user.username,
                email: user.email,
            })
            .collect();

        let response = GetUsersResponse { users: grpc_users };

        Ok(Response::new(response))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        match self.service.find_by_id(req.id).await {
            Some(user) => {
                info!("gRPC: Fetched user {}", req.id);
                let grpc_user = User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                };

                let response = GetUserResponse {
                    user: Some(grpc_user),
                };

                Ok(Response::new(response))
            }
            None => {
                info!("gRPC: User {} not found", req.id);
                Err(Status::not_found("ユーザーが見つかりません"))
            }
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();

        let create_request = crate::application::dto::user_dto::CreateUserRequest {
            username: req.username,
            email: req.email,
        };

        let new_user = self.service.create(create_request).await;
        info!("gRPC: Created user with id {}", new_user.id);

        let grpc_user = User {
            id: new_user.id,
            username: new_user.username,
            email: new_user.email,
        };

        let response = CreateUserResponse {
            user: Some(grpc_user),
        };

        Ok(Response::new(response))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();

        let update_request = crate::application::dto::user_dto::UpdateUserRequest {
            username: req.username,
            email: req.email,
        };

        match self.service.update(req.id, update_request).await {
            Some(updated_user) => {
                info!("gRPC: Updated user {}", req.id);
                let grpc_user = User {
                    id: updated_user.id,
                    username: updated_user.username,
                    email: updated_user.email,
                };

                let response = UpdateUserResponse {
                    user: Some(grpc_user),
                };

                Ok(Response::new(response))
            }
            None => {
                info!("gRPC: User {} not found for update", req.id);
                Err(Status::not_found("ユーザーが見つかりません"))
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();

        let success = self.service.delete(req.id).await;
        info!("gRPC: Delete user {} result: {}", req.id, success);

        let response = DeleteUserResponse { success };

        Ok(Response::new(response))
    }
}
