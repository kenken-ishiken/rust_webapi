use actix_web::{web, HttpResponse, Responder};
use tracing::info;
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
        info!("Fetched {} users", users.len());
        HttpResponse::Ok().json(users)
    }

    pub async fn get_user(
        data: web::Data<UserHandler>, 
        path: web::Path<u64>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        match data.service.find_by_id(user_id).await {
            Some(user) => {
                info!("Fetched user {}", user_id);
                HttpResponse::Ok().json(user)
            },
            None => {
                info!("User {} not found", user_id);
                HttpResponse::NotFound().json("ユーザーが見つかりません")
            },
        }
    }

    pub async fn create_user(
        data: web::Data<UserHandler>,
        user: web::Json<CreateUserRequest>,
    ) -> impl Responder {
        let new_user = data.service.create(user.into_inner()).await;
        info!("Created user with id {}", new_user.id);
        HttpResponse::Created().json(new_user)
    }

    pub async fn update_user(
        data: web::Data<UserHandler>,
        path: web::Path<u64>,
        user: web::Json<UpdateUserRequest>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        match data.service.update(user_id, user.into_inner()).await {
            Some(updated_user) => {
                info!("Updated user {}", user_id);
                HttpResponse::Ok().json(updated_user)
            },
            None => {
                info!("User {} not found for update", user_id);
                HttpResponse::NotFound().json("ユーザーが見つかりません")
            },
        }
    }

    pub async fn delete_user(
        data: web::Data<UserHandler>,
        path: web::Path<u64>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        if data.service.delete(user_id).await {
            info!("Deleted user {}", user_id);
            HttpResponse::Ok().json("ユーザーを削除しました")
        } else {
            info!("User {} not found for deletion", user_id);
            HttpResponse::NotFound().json("ユーザーが見つかりません")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test};
    use mockall::predicate::*;
    use mockall::mock;
    use domain::repository::user_repository::UserRepository;
    use domain::model::user::User;
    
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

    #[actix_web::test]
    async fn test_get_users() {
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_find_all()
            .return_once(|| {
                vec![
                    User { id: 1, username: "user1".to_string(), email: "user1@example.com".to_string() },
                    User { id: 2, username: "user2".to_string(), email: "user2@example.com".to_string() },
                ]
            });
        
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        
        let resp = UserHandler::get_users(handler).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_get_user_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| Some(User {
                id: 1,
                username: "user1".to_string(),
                email: "user1@example.com".to_string(),
            }));
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let path = web::Path::from(1u64);
        
        let resp = UserHandler::get_user(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_get_user_not_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let path = web::Path::from(999u64);
        
        let resp = UserHandler::get_user(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
    
    #[actix_web::test]
    async fn test_create_user() {
        let req = CreateUserRequest {
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
        };
        
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_create()
            .withf(|user| {
                user.username == "newuser" && user.email == "newuser@example.com"
            })
            .return_once(|_| User {
                id: 42,
                username: "newuser".to_string(),
                email: "newuser@example.com".to_string(),
            });
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let json_req = web::Json(req);
        
        let resp = UserHandler::create_user(handler, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
    
    #[actix_web::test]
    async fn test_update_user_found() {
        let req = UpdateUserRequest {
            username: Some("updateduser".to_string()),
            email: Some("updated@example.com".to_string()),
        };
        
        let updated_user = User {
            id: 1,
            username: "updateduser".to_string(),
            email: "updated@example.com".to_string(),
        };
        
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| Some(User {
                id: 1,
                username: "originaluser".to_string(),
                email: "original@example.com".to_string(),
            }));
            
        mock_repo.expect_update()
            .withf(|user| {
                user.id == 1 && 
                user.username == "updateduser" && 
                user.email == "updated@example.com"
            })
            .return_once(move |_| Some(updated_user.clone()));
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let path = web::Path::from(1u64);
        let json_req = web::Json(req);
        
        let resp = UserHandler::update_user(handler, path, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_update_user_not_found() {
        let req = UpdateUserRequest {
            username: Some("updateduser".to_string()),
            email: None,
        };
        
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let path = web::Path::from(999u64);
        let json_req = web::Json(req);
        
        let resp = UserHandler::update_user(handler, path, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
    
    #[actix_web::test]
    async fn test_delete_user_success() {
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let path = web::Path::from(1u64);
        
        let resp = UserHandler::delete_user(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_delete_user_not_found() {
        let mut mock_repo = MockUserRep::new();
        mock_repo.expect_delete()
            .with(eq(999u64))
            .return_once(|_| false);
            
        let service = Arc::new(UserService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(UserHandler::new(service));
        let path = web::Path::from(999u64);
        
        let resp = UserHandler::delete_user(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());
        
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}