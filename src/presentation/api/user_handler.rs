use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::application::dto::user_dto::{CreateUserRequest, UpdateUserRequest};
use crate::application::service::user_service::UserService;

pub struct UserHandler {
    service: Arc<UserService>,
}

impl UserHandler {
    pub fn new(service: Arc<UserService>) -> Self {
        Self { service }
    }

    pub async fn get_users(data: web::Data<UserHandler>) -> impl Responder {
        let users = data.service.find_all().await;
        HttpResponse::Ok().json(users)
    }

    pub async fn get_user(
        data: web::Data<UserHandler>, 
        path: web::Path<u64>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        match data.service.find_by_id(user_id).await {
            Some(user) => HttpResponse::Ok().json(user),
            None => HttpResponse::NotFound().json("ユーザーが見つかりません"),
        }
    }

    pub async fn create_user(
        data: web::Data<UserHandler>,
        user: web::Json<CreateUserRequest>,
    ) -> impl Responder {
        let new_user = data.service.create(user.into_inner()).await;
        HttpResponse::Created().json(new_user)
    }

    pub async fn update_user(
        data: web::Data<UserHandler>,
        path: web::Path<u64>,
        user: web::Json<UpdateUserRequest>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        match data.service.update(user_id, user.into_inner()).await {
            Some(updated_user) => HttpResponse::Ok().json(updated_user),
            None => HttpResponse::NotFound().json("ユーザーが見つかりません"),
        }
    }

    pub async fn delete_user(
        data: web::Data<UserHandler>,
        path: web::Path<u64>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        if data.service.delete(user_id).await {
            HttpResponse::Ok().json("ユーザーを削除しました")
        } else {
            HttpResponse::NotFound().json("ユーザーが見つかりません")
        }
    }
}