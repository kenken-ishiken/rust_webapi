use actix_web::{web, HttpResponse, Responder, Result as ActixResult};
use std::sync::Arc;
use tracing::info;

use crate::application::dto::item_dto::{
    CreateItemRequest, UpdateItemRequest, BatchDeleteRequest, 
    BatchDeleteResponse, DeletionValidationResponse, DeletionLogResponse
};
use crate::application::service::item_service::ItemService;
use crate::infrastructure::auth::middleware::KeycloakUser;

pub struct ItemHandler {
    service: Arc<ItemService>,
}

impl ItemHandler {
    pub fn new(service: Arc<ItemService>) -> Self {
        Self { service }
    }

    pub async fn index() -> impl Responder {
        HttpResponse::Ok().json("Rust WebAPI サーバーが稼働中です")
    }

    pub async fn get_items(
        data: web::Data<ItemHandler>,
        user: KeycloakUser
    ) -> ActixResult<impl Responder> {
        // 認証済みユーザー情報をログに出力
        info!("ユーザー {} がアイテム一覧を取得しました", user.claims.preferred_username);

        let items = data.service.find_all().await?;
        Ok(HttpResponse::Ok().json(items))
    }

    pub async fn get_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        let item = data.service.find_by_id(item_id).await?;
        info!("Fetched item {}", item_id);
        Ok(HttpResponse::Ok().json(item))
    }

    pub async fn create_item(
        data: web::Data<ItemHandler>,
        item: web::Json<CreateItemRequest>,
    ) -> ActixResult<impl Responder> {
        let new_item = data.service.create(item.into_inner()).await?;
        info!("Created item {}", new_item.id);
        Ok(HttpResponse::Created().json(new_item))
    }

    pub async fn update_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
        item: web::Json<UpdateItemRequest>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        let updated_item = data.service.update(item_id, item.into_inner()).await?;
        info!("Updated item {}", item_id);
        Ok(HttpResponse::Ok().json(updated_item))
    }

    pub async fn delete_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        data.service.delete(item_id).await?;
        info!("Deleted item {}", item_id);
        Ok(HttpResponse::Ok().json("アイテムを削除しました"))
    }
    
    // New handlers for product deletion API
    
    pub async fn logical_delete_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        data.service.logical_delete(item_id).await?;
        info!("Logically deleted item {}", item_id);
        Ok(HttpResponse::Ok().json("アイテムを論理削除しました"))
    }
    
    pub async fn physical_delete_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        data.service.physical_delete(item_id).await?;
        info!("Physically deleted item {}", item_id);
        Ok(HttpResponse::Ok().json("アイテムを物理削除しました"))
    }
    
    pub async fn restore_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        data.service.restore(item_id).await?;
        info!("Restored item {}", item_id);
        Ok(HttpResponse::Ok().json("アイテムを復元しました"))
    }
    
    pub async fn validate_item_deletion(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        let validation = data.service.validate_deletion(item_id).await?;
        info!("Validated deletion for item {}", item_id);
        Ok(HttpResponse::Ok().json(validation))
    }
    
    pub async fn batch_delete_items(
        data: web::Data<ItemHandler>,
        req: web::Json<BatchDeleteRequest>,
    ) -> ActixResult<impl Responder> {
        let result = data.service.batch_delete(req.into_inner()).await?;
        info!("Batch deleted {} items", result.successful_ids.len());
        Ok(HttpResponse::Ok().json(result))
    }
    
    pub async fn get_deleted_items(
        data: web::Data<ItemHandler>,
    ) -> ActixResult<impl Responder> {
        let deleted_items = data.service.find_deleted().await?;
        info!("Fetched {} deleted items", deleted_items.len());
        Ok(HttpResponse::Ok().json(deleted_items))
    }
    
    pub async fn get_item_deletion_log(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> ActixResult<impl Responder> {
        let item_id = path.into_inner();
        let logs = data.service.get_deletion_logs(Some(item_id)).await?;
        info!("Fetched {} deletion logs for item {}", logs.len(), item_id);
        Ok(HttpResponse::Ok().json(logs))
    }
    
    pub async fn get_deletion_logs(
        data: web::Data<ItemHandler>,
    ) -> ActixResult<impl Responder> {
        let logs = data.service.get_deletion_logs(None).await?;
        info!("Fetched {} deletion logs", logs.len());
        Ok(HttpResponse::Ok().json(logs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, http::StatusCode};
    use crate::application::service::item_service::ItemService;
    use domain::model::item::{Item, DeletionValidation, RelatedDataCount, DeletionLog, DeletionType};
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;
    use crate::infrastructure::auth::keycloak::KeycloakClaims;
    use domain::repository::item_repository::ItemRepository;
    use chrono::Utc;

    mock! {
        ItemRep {}
        #[async_trait::async_trait]
        impl ItemRepository for ItemRep {
            async fn find_all(&self) -> Vec<Item>;
            async fn find_by_id(&self, id: u64) -> Option<Item>;
            async fn create(&self, item: Item) -> Item;
            async fn update(&self, item: Item) -> Option<Item>;
            async fn delete(&self, id: u64) -> bool;
            async fn logical_delete(&self, id: u64) -> bool;
            async fn physical_delete(&self, id: u64) -> bool;
            async fn restore(&self, id: u64) -> bool;
            async fn find_deleted(&self) -> Vec<Item>;
            async fn validate_deletion(&self, id: u64) -> DeletionValidation;
            async fn batch_delete(&self, ids: Vec<u64>, is_physical: bool) -> Vec<u64>;
            async fn get_deletion_logs(&self, item_id: Option<u64>) -> Vec<DeletionLog>;
        }
    }

    impl KeycloakUser {
        fn mock() -> Self {
            Self {
                claims: KeycloakClaims {
                    sub: "test-user-id".to_string(),
                    preferred_username: "test-user".to_string(),
                    email: "test@example.com".to_string(),
                    name: "Test User".to_string(),
                    given_name: "Test".to_string(),
                    family_name: "User".to_string(),
                    exp: 0,
                    iat: 0,
                    auth_time: 0,
                    jti: "test-jti".to_string(),
                    iss: "test-issuer".to_string(),
                    aud: "test-audience".to_string(),
                    typ: "Bearer".to_string(),
                    azp: "test-azp".to_string(),
                    session_state: "test-session".to_string(),
                    acr: "1".to_string(),
                    realm_access: crate::infrastructure::auth::keycloak::RealmAccess {
                        roles: vec!["user".to_string()],
                    },
                    resource_access: crate::infrastructure::auth::keycloak::ResourceAccess {
                        account: crate::infrastructure::auth::keycloak::Account {
                            roles: vec!["manage-account".to_string()],
                        },
                    },
                    scope: "openid profile email".to_string(),
                    sid: "test-sid".to_string(),
                    email_verified: true,
                },
            }
        }
    }

    #[actix_web::test]
    async fn test_index() {
        let resp = ItemHandler::index().await;

        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_items() {
        let items = vec![
            Item {
                id: 1,
                name: "Item 1".to_string(),
                description: Some("Description 1".to_string()),
                deleted: false,
                deleted_at: None,
            },
            Item {
                id: 2,
                name: "Item 2".to_string(),
                description: None,
                deleted: false,
                deleted_at: None,
            },
        ];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_all()
            .return_once(move || items.clone());

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let user = KeycloakUser::mock();

        let resp = ItemHandler::get_items(handler, user).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_item_found() {
        let item = Item {
            id: 1,
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(item.clone()));

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);

        let resp = ItemHandler::get_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_item_not_found() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(999u64);

        let resp = ItemHandler::get_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_create_item() {
        let req = CreateItemRequest {
            name: "New Item".to_string(),
            description: Some("New Description".to_string()),
        };

        let created_item = Item {
            id: 1,
            name: "New Item".to_string(),
            description: Some("New Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_create()
            .return_once(move |_| created_item.clone());

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let json_req = web::Json(req);

        let resp = ItemHandler::create_item(handler, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_update_item_found() {
        let req = UpdateItemRequest {
            name: Some("Updated Item".to_string()),
            description: Some("Updated Description".to_string()),
        };

        let updated_item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| Some(Item {
                id: 1,
                name: "Original Item".to_string(),
                description: None,
                deleted: false,
                deleted_at: None,
            }));

        mock_repo.expect_update()
            .return_once(move |_| Some(updated_item.clone()));

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);
        let json_req = web::Json(req);

        let resp = ItemHandler::update_item(handler, path, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_update_item_not_found() {
        let req = UpdateItemRequest {
            name: Some("Updated Item".to_string()),
            description: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(999u64);
        let json_req = web::Json(req);

        let resp = ItemHandler::update_item(handler, path, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_delete_item_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);

        let resp = ItemHandler::delete_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_delete_item_not_found() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_delete()
            .with(eq(999u64))
            .return_once(|_| false);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(999u64);

        let resp = ItemHandler::delete_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
    
    // Tests for new handlers
    
    #[actix_web::test]
    async fn test_logical_delete_item_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_logical_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);

        let resp = ItemHandler::logical_delete_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_physical_delete_item_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_physical_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);

        let resp = ItemHandler::physical_delete_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_restore_item_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_restore()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);

        let resp = ItemHandler::restore_item(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_validate_item_deletion() {
        let validation = DeletionValidation {
            can_delete: true,
            related_data: RelatedDataCount {
                related_orders: 0,
                related_reviews: 0,
                related_categories: 0,
            },
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| Some(Item {
                id: 1,
                name: "Test Item".to_string(),
                description: None,
                deleted: false,
                deleted_at: None,
            }));
            
        mock_repo.expect_validate_deletion()
            .with(eq(1u64))
            .return_once(move |_| validation);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let path = web::Path::from(1u64);

        let resp = ItemHandler::validate_item_deletion(handler, path).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_batch_delete_items() {
        let req = BatchDeleteRequest {
            ids: vec![1, 2, 3],
            is_physical: Some(false),
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_batch_delete()
            .with(eq(vec![1, 2, 3]), eq(false))
            .return_once(move |_, _| vec![1, 3]);

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));
        let json_req = web::Json(req);

        let resp = ItemHandler::batch_delete_items(handler, json_req).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_get_deleted_items() {
        let deleted_items = vec![
            Item {
                id: 1,
                name: "Deleted Item 1".to_string(),
                description: Some("Description 1".to_string()),
                deleted: true,
                deleted_at: Some(Utc::now()),
            },
            Item {
                id: 2,
                name: "Deleted Item 2".to_string(),
                description: None,
                deleted: true,
                deleted_at: Some(Utc::now()),
            },
        ];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_deleted()
            .return_once(move || deleted_items.clone());

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));

        let resp = ItemHandler::get_deleted_items(handler).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_get_deletion_logs() {
        let now = Utc::now();
        let logs = vec![
            DeletionLog {
                id: 1,
                item_id: 1,
                item_name: "Item 1".to_string(),
                deletion_type: DeletionType::Logical,
                deleted_at: now,
                deleted_by: "test_user".to_string(),
            },
            DeletionLog {
                id: 2,
                item_id: 2,
                item_name: "Item 2".to_string(),
                deletion_type: DeletionType::Physical,
                deleted_at: now,
                deleted_by: "test_user".to_string(),
            },
        ];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_get_deletion_logs()
            .with(eq(None))
            .return_once(move |_| logs.clone());

        let service = Arc::new(ItemService::new(Arc::new(mock_repo)));
        let handler = web::Data::new(ItemHandler::new(service));

        let resp = ItemHandler::get_deletion_logs(handler).await;
        let resp = resp.respond_to(&test::TestRequest::default().to_http_request());

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
