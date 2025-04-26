use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use tracing::info;

use crate::application::dto::item_dto::{CreateItemRequest, UpdateItemRequest};
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
    ) -> impl Responder {
        // 認証済みユーザー情報をログに出力
        info!("ユーザー {} がアイテム一覧を取得しました", user.claims.preferred_username);

        let items = data.service.find_all().await;
        HttpResponse::Ok().json(items)
    }

    pub async fn get_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> impl Responder {
        let item_id = path.into_inner();
        match data.service.find_by_id(item_id).await {
            Some(item) => {
                info!("Fetched item {}", item_id);
                HttpResponse::Ok().json(item)
            },
            None => {
                info!("Item {} not found", item_id);
                HttpResponse::NotFound().json("アイテムが見つかりません")
            },
        }
    }

    pub async fn create_item(
        data: web::Data<ItemHandler>,
        item: web::Json<CreateItemRequest>,
    ) -> impl Responder {
        let new_item = data.service.create(item.into_inner()).await;
        info!("Created item {}", new_item.id);
        HttpResponse::Created().json(new_item)
    }

    pub async fn update_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
        item: web::Json<UpdateItemRequest>,
    ) -> impl Responder {
        let item_id = path.into_inner();
        match data.service.update(item_id, item.into_inner()).await {
            Some(updated_item) => {
                info!("Updated item {}", item_id);
                HttpResponse::Ok().json(updated_item)
            },
            None => {
                info!("Item {} not found for update", item_id);
                HttpResponse::NotFound().json("アイテムが見つかりません")
            },
        }
    }

    pub async fn delete_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> impl Responder {
        let item_id = path.into_inner();
        if data.service.delete(item_id).await {
            info!("Deleted item {}", item_id);
            HttpResponse::Ok().json("アイテムを削除しました")
        } else {
            info!("Item {} not found for deletion", item_id);
            HttpResponse::NotFound().json("アイテムが見つかりません")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, http::StatusCode};
    use crate::application::service::item_service::ItemService;
    use domain::model::item::Item;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;
    use crate::infrastructure::auth::keycloak::KeycloakClaims;
    use domain::repository::item_repository::ItemRepository;

    mock! {
        ItemRep {}
        #[async_trait::async_trait]
        impl ItemRepository for ItemRep {
            async fn find_all(&self) -> Vec<Item>;
            async fn find_by_id(&self, id: u64) -> Option<Item>;
            async fn create(&self, item: Item) -> Item;
            async fn update(&self, item: Item) -> Option<Item>;
            async fn delete(&self, id: u64) -> bool;
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
            },
            Item {
                id: 2,
                name: "Item 2".to_string(),
                description: None,
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
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| Some(Item {
                id: 1,
                name: "Original Item".to_string(),
                description: None,
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
}
