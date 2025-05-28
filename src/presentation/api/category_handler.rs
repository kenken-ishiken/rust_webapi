use actix_web::{web, HttpResponse, Responder, Result as ActixResult};
use tracing::{info, error};
use std::sync::Arc;

use crate::application::dto::category_dto::{
    CreateCategoryRequest, UpdateCategoryRequest, MoveCategoryRequest,
    CategoryQueryParams, CategoryErrorResponse
};
use crate::application::service::category_service::CategoryService;
use crate::infrastructure::auth::middleware::KeycloakUser;

pub struct CategoryHandler {
    service: Arc<CategoryService>,
}

impl CategoryHandler {
    pub fn new(service: Arc<CategoryService>) -> Self {
        Self { service }
    }

    pub async fn get_categories(
        data: web::Data<CategoryHandler>,
        query: web::Query<CategoryQueryParams>,
    ) -> ActixResult<impl Responder> {
        let include_inactive = query.include_inactive.unwrap_or(false);
        
        info!("Fetching categories with include_inactive: {}", include_inactive);
        
        match &query.parent_id {
            Some(parent_id) => {
                let response = data.service.find_by_parent_id(Some(parent_id.clone()), include_inactive).await;
                info!("Fetched {} categories for parent {}", response.total, parent_id);
                Ok(HttpResponse::Ok().json(response))
            }
            None => {
                let response = data.service.find_all(include_inactive).await;
                info!("Fetched {} categories", response.total);
                Ok(HttpResponse::Ok().json(response))
            }
        }
    }

    pub async fn get_category(
        data: web::Data<CategoryHandler>,
        path: web::Path<String>,
    ) -> ActixResult<impl Responder> {
        let category_id = path.into_inner();
        
        match data.service.find_by_id(&category_id).await {
            Ok(category) => {
                info!("Fetched category {}", category_id);
                Ok(HttpResponse::Ok().json(category))
            }
            Err(error) => {
                error!("Category {} not found: {}", category_id, error);
                let error_response: CategoryErrorResponse = error.into();
                match error_response.code.as_str() {
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    pub async fn get_category_children(
        data: web::Data<CategoryHandler>,
        path: web::Path<String>,
        query: web::Query<CategoryQueryParams>,
    ) -> ActixResult<impl Responder> {
        let category_id = path.into_inner();
        let include_inactive = query.include_inactive.unwrap_or(false);
        
        let response = data.service.find_children(&category_id, include_inactive).await;
        info!("Fetched {} children for category {}", response.total, category_id);
        Ok(HttpResponse::Ok().json(response))
    }

    pub async fn get_category_path(
        data: web::Data<CategoryHandler>,
        path: web::Path<String>,
    ) -> ActixResult<impl Responder> {
        let category_id = path.into_inner();
        
        match data.service.find_path(&category_id).await {
            Ok(path_response) => {
                info!("Fetched path for category {}, depth: {}", category_id, path_response.depth);
                Ok(HttpResponse::Ok().json(path_response))
            }
            Err(error) => {
                error!("Failed to get path for category {}: {}", category_id, error);
                let error_response: CategoryErrorResponse = error.into();
                match error_response.code.as_str() {
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    pub async fn get_category_tree(
        data: web::Data<CategoryHandler>,
        query: web::Query<CategoryQueryParams>,
    ) -> ActixResult<impl Responder> {
        let include_inactive = query.include_inactive.unwrap_or(false);
        
        let response = data.service.find_tree(include_inactive).await;
        info!("Fetched category tree with {} root categories", response.tree.len());
        Ok(HttpResponse::Ok().json(response))
    }

    pub async fn create_category(
        data: web::Data<CategoryHandler>,
        category: web::Json<CreateCategoryRequest>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        match data.service.create(category.into_inner()).await {
            Ok(created_category) => {
                info!("Created category with id {}", created_category.id);
                Ok(HttpResponse::Created().json(created_category))
            }
            Err(error) => {
                error!("Failed to create category: {}", error);
                let error_response: CategoryErrorResponse = error.into();
                match error_response.code.as_str() {
                    "CATEGORY_NAME_DUPLICATE" => Ok(HttpResponse::Conflict().json(error_response)),
                    "CATEGORY_INVALID_NAME" | "CATEGORY_INVALID_SORT_ORDER" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    "CATEGORY_MAX_DEPTH_EXCEEDED" => Ok(HttpResponse::BadRequest().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    pub async fn update_category(
        data: web::Data<CategoryHandler>,
        path: web::Path<String>,
        category: web::Json<UpdateCategoryRequest>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let category_id = path.into_inner();
        
        match data.service.update(&category_id, category.into_inner()).await {
            Ok(updated_category) => {
                info!("Updated category {}", category_id);
                Ok(HttpResponse::Ok().json(updated_category))
            }
            Err(error) => {
                error!("Failed to update category {}: {}", category_id, error);
                let error_response: CategoryErrorResponse = error.into();
                match error_response.code.as_str() {
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "CATEGORY_NAME_DUPLICATE" => Ok(HttpResponse::Conflict().json(error_response)),
                    "CATEGORY_INVALID_NAME" | "CATEGORY_INVALID_SORT_ORDER" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    pub async fn delete_category(
        data: web::Data<CategoryHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let category_id = path.into_inner();
        
        match data.service.delete(&category_id).await {
            Ok(_) => {
                info!("Deleted category {}", category_id);
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "カテゴリを削除しました"
                })))
            }
            Err(error) => {
                error!("Failed to delete category {}: {}", category_id, error);
                let error_response: CategoryErrorResponse = error.into();
                match error_response.code.as_str() {
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "CATEGORY_HAS_CHILDREN" | "CATEGORY_HAS_PRODUCTS" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    pub async fn move_category(
        data: web::Data<CategoryHandler>,
        path: web::Path<String>,
        move_req: web::Json<MoveCategoryRequest>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let category_id = path.into_inner();
        
        match data.service.move_category(&category_id, move_req.into_inner()).await {
            Ok(moved_category) => {
                info!("Moved category {}", category_id);
                Ok(HttpResponse::Ok().json(moved_category))
            }
            Err(error) => {
                error!("Failed to move category {}: {}", category_id, error);
                let error_response: CategoryErrorResponse = error.into();
                match error_response.code.as_str() {
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "CATEGORY_CIRCULAR_REFERENCE" | "CATEGORY_MAX_DEPTH_EXCEEDED" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }
}

// Configure category routes
pub fn configure_category_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/categories")
            .route("", web::get().to(CategoryHandler::get_categories))
            .route("", web::post().to(CategoryHandler::create_category))
            .route("/tree", web::get().to(CategoryHandler::get_category_tree))
            .route("/{id}", web::get().to(CategoryHandler::get_category))
            .route("/{id}", web::put().to(CategoryHandler::update_category))
            .route("/{id}", web::delete().to(CategoryHandler::delete_category))
            .route("/{id}/children", web::get().to(CategoryHandler::get_category_children))
            .route("/{id}/path", web::get().to(CategoryHandler::get_category_path))
            .route("/{id}/move", web::put().to(CategoryHandler::move_category))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, http::StatusCode};
    use std::sync::Arc;
    use crate::app_domain::repository::category_repository::MockCategoryRepository;
    use crate::app_domain::model::category::{Category, CategoryError};
    use crate::application::service::category_service::CategoryService;
    use mockall::predicate::*;
    use chrono::Utc;

    fn create_test_category() -> Category {
        Category {
            id: "cat_123".to_string(),
            name: "Electronics".to_string(),
            description: Some("Electronic devices".to_string()),
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[actix_web::test]
    async fn test_get_category_success() {
        let mut mock_repo = MockCategoryRepository::new();
        let category = create_test_category();
        
        mock_repo
            .expect_find_by_id()
            .with(eq("cat_123"))
            .return_once(move |_| Some(category));
        
        let service = Arc::new(CategoryService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(CategoryHandler::new(service));

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/categories/{id}", web::get().to(CategoryHandler::get_category))
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/categories/cat_123")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_category_not_found() {
        let mut mock_repo = MockCategoryRepository::new();
        
        mock_repo
            .expect_find_by_id()
            .with(eq("cat_999"))
            .return_once(|_| None);
        
        let service = Arc::new(CategoryService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(CategoryHandler::new(service));

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/categories/{id}", web::get().to(CategoryHandler::get_category))
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/categories/cat_999")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_get_categories_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let categories = vec![create_test_category()];
        
        mock_repo
            .expect_find_all()
            .with(eq(false))
            .return_once(move |_| categories);
        
        mock_repo
            .expect_count_children()
            .with(eq("cat_123"))
            .return_once(|_| 0);
        
        let service = Arc::new(CategoryService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(CategoryHandler::new(service));

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/categories", web::get().to(CategoryHandler::get_categories))
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/categories")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_category_tree_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        mock_repo
            .expect_find_tree()
            .with(eq(false))
            .return_once(|_| vec![]);
        
        let service = Arc::new(CategoryService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(CategoryHandler::new(service));

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/categories/tree", web::get().to(CategoryHandler::get_category_tree))
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/categories/tree")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_category_children_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let child_category = Category {
            id: "cat_456".to_string(),
            name: "Smartphones".to_string(),
            description: Some("Smart devices".to_string()),
            parent_id: Some("cat_123".to_string()),
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        mock_repo
            .expect_find_by_parent_id()
            .with(eq(Some("cat_123".to_string())), eq(false))
            .return_once(move |_, _| vec![child_category]);
        
        mock_repo
            .expect_count_children()
            .with(eq("cat_456"))
            .return_once(|_| 0);
        
        let service = Arc::new(CategoryService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(CategoryHandler::new(service));

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/categories/{id}/children", web::get().to(CategoryHandler::get_category_children))
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/categories/cat_123/children")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}