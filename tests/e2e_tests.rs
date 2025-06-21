mod helpers;

use actix_web::{test, web, App, HttpResponse};
use helpers::test_environment::TestRepositoryFactory;
use serde_json::{json, Value};
use std::sync::Arc;

use rust_webapi::application::service::item_service::ItemService;
use rust_webapi::application::service::user_service::UserService;

// Test-specific handlers that work directly with services (bypassing authentication)
async fn test_get_items(item_service: web::Data<Arc<ItemService>>) -> impl actix_web::Responder {
    let items = item_service.find_all().await;
    match items {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(_) => HttpResponse::InternalServerError().json("Error fetching items"),
    }
}

async fn test_create_item(
    item_service: web::Data<Arc<ItemService>>,
    item: web::Json<Value>,
) -> impl actix_web::Responder {
    use rust_webapi::application::dto::item_dto::CreateItemRequest;

    let create_request = CreateItemRequest {
        name: item["name"].as_str().unwrap_or("").to_string(),
        description: item["description"].as_str().map(|s| s.to_string()),
    };

    let result = item_service.create(create_request).await;
    match result {
        Ok(item) => HttpResponse::Created().json(item),
        Err(_) => HttpResponse::InternalServerError().json("Error creating item"),
    }
}

async fn test_get_item(
    item_service: web::Data<Arc<ItemService>>,
    path: web::Path<u64>,
) -> impl actix_web::Responder {
    let item_id = path.into_inner();
    let result = item_service.find_by_id(item_id).await;
    match result {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::NotFound().json("Item not found"),
    }
}

async fn test_update_item(
    item_service: web::Data<Arc<ItemService>>,
    path: web::Path<u64>,
    item: web::Json<Value>,
) -> impl actix_web::Responder {
    use rust_webapi::application::dto::item_dto::UpdateItemRequest;

    let item_id = path.into_inner();
    let update_request = UpdateItemRequest {
        name: item["name"].as_str().map(|s| s.to_string()),
        description: item["description"].as_str().map(|s| s.to_string()),
    };

    let result = item_service.update(item_id, update_request).await;
    match result {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::NotFound().json("Item not found"),
    }
}

async fn test_delete_item(
    item_service: web::Data<Arc<ItemService>>,
    path: web::Path<u64>,
) -> impl actix_web::Responder {
    let item_id = path.into_inner();
    let result = item_service.delete(item_id).await;
    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().json("Error deleting item"),
    }
}

async fn test_get_users(user_service: web::Data<Arc<UserService>>) -> impl actix_web::Responder {
    let users = user_service.find_all().await;
    HttpResponse::Ok().json(users)
}

async fn test_create_user(
    user_service: web::Data<Arc<UserService>>,
    user: web::Json<Value>,
) -> impl actix_web::Responder {
    use rust_webapi::application::dto::user_dto::CreateUserRequest;

    let create_request = CreateUserRequest {
        username: user["username"].as_str().unwrap_or("").to_string(),
        email: user["email"].as_str().unwrap_or("").to_string(),
    };

    let user = user_service.create(create_request).await;
    HttpResponse::Created().json(user)
}

async fn test_get_user(
    user_service: web::Data<Arc<UserService>>,
    path: web::Path<u64>,
) -> impl actix_web::Responder {
    let user_id = path.into_inner();
    match user_service.find_by_id(user_id).await {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().json("User not found"),
    }
}

async fn test_update_user(
    user_service: web::Data<Arc<UserService>>,
    path: web::Path<u64>,
    user: web::Json<Value>,
) -> impl actix_web::Responder {
    use rust_webapi::application::dto::user_dto::UpdateUserRequest;

    let user_id = path.into_inner();
    let update_request = UpdateUserRequest {
        username: user["username"].as_str().map(|s| s.to_string()),
        email: user["email"].as_str().map(|s| s.to_string()),
    };

    match user_service.update(user_id, update_request).await {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().json("User not found"),
    }
}

async fn test_delete_user(
    user_service: web::Data<Arc<UserService>>,
    path: web::Path<u64>,
) -> impl actix_web::Responder {
    let user_id = path.into_inner();
    match user_service.delete(user_id).await {
        true => HttpResponse::Ok().json("User deleted"),
        false => HttpResponse::NotFound().json("User not found"),
    }
}

/// Creates a simple test application for E2E testing
fn create_test_app(
    item_service: Arc<ItemService>,
    user_service: Arc<UserService>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(Arc::clone(&item_service)))
        .app_data(web::Data::new(Arc::clone(&user_service)))
        .service(
            web::scope("/api")
                // Health check endpoint
                .route("/health", web::get().to(|| async { "OK" }))
                // Items endpoints (without authentication for testing)
                .route("/items", web::get().to(test_get_items))
                .route("/items", web::post().to(test_create_item))
                .route("/items/{id}", web::get().to(test_get_item))
                .route("/items/{id}", web::put().to(test_update_item))
                .route("/items/{id}", web::delete().to(test_delete_item))
                // Users endpoints
                .route("/users", web::get().to(test_get_users))
                .route("/users", web::post().to(test_create_user))
                .route("/users/{id}", web::get().to(test_get_user))
                .route("/users/{id}", web::put().to(test_update_user))
                .route("/users/{id}", web::delete().to(test_delete_user)),
        )
}

#[tokio::test]
async fn test_e2e_health_check() {
    let factory = TestRepositoryFactory::new().await;

    let item_repository = factory.create_item_repository_for_service();
    let user_repository = factory.create_user_repository();

    // Use the repository trait directly to avoid casting issues
    let item_service = Arc::new(ItemService::new(item_repository));
    let user_service = Arc::new(UserService::new(user_repository));

    let app = test::init_service(create_test_app(item_service.clone(), user_service.clone())).await;

    let req = test::TestRequest::get().uri("/api/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    assert_eq!(body, "OK");

    println!("✅ E2E Health check test passed");
}

#[tokio::test]
async fn test_e2e_items_crud_workflow() {
    let factory = TestRepositoryFactory::new().await;

    let item_repository = factory.create_item_repository_for_service();
    let user_repository = factory.create_user_repository();

    let item_service = Arc::new(ItemService::new(item_repository));
    let user_service = Arc::new(UserService::new(user_repository));

    let app = test::init_service(create_test_app(item_service.clone(), user_service.clone())).await;

    // Test 1: Create an item
    let create_request = json!({
        "name": "E2E Test Item",
        "description": "Item created via E2E test"
    });

    let req = test::TestRequest::post()
        .uri("/api/items")
        .set_json(&create_request)
        .to_request();
    let resp = test::call_service(&app, req).await;

    if resp.status() != 201 {
        let status = resp.status();
        let body = test::read_body(resp).await;
        println!(
            "Expected 201, got {} with body: {:?}",
            status,
            std::str::from_utf8(&body)
        );
        panic!("Item creation failed");
    }

    assert_eq!(resp.status(), 201);
    let created_item: Value = test::read_body_json(resp).await;
    let item_id = created_item["id"].as_u64().unwrap();

    assert_eq!(created_item["name"], "E2E Test Item");
    assert_eq!(created_item["description"], "Item created via E2E test");
    println!("✅ E2E Item creation test passed - ID: {}", item_id);

    // Test 2: Get the created item
    let req = test::TestRequest::get()
        .uri(&format!("/api/items/{}", item_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let fetched_item: Value = test::read_body_json(resp).await;
    assert_eq!(fetched_item["id"], item_id);
    assert_eq!(fetched_item["name"], "E2E Test Item");
    println!("✅ E2E Item retrieval test passed");

    // Test 3: Update the item
    let update_request = json!({
        "name": "Updated E2E Item",
        "description": "Updated description via E2E test"
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/items/{}", item_id))
        .set_json(&update_request)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let updated_item: Value = test::read_body_json(resp).await;
    assert_eq!(updated_item["name"], "Updated E2E Item");
    assert_eq!(
        updated_item["description"],
        "Updated description via E2E test"
    );
    println!("✅ E2E Item update test passed");

    // Test 4: Get all items (should contain our item)
    let req = test::TestRequest::get().uri("/api/items").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let items: Value = test::read_body_json(resp).await;
    let items_array = items.as_array().unwrap();
    assert!(!items_array.is_empty());

    let found_item = items_array.iter().find(|item| item["id"] == item_id);
    assert!(found_item.is_some());
    println!("✅ E2E Items list test passed");

    // Test 5: Delete the item
    let req = test::TestRequest::delete()
        .uri(&format!("/api/items/{}", item_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);
    println!("✅ E2E Item deletion test passed");

    // Test 6: Verify item is deleted (should return 404)
    let req = test::TestRequest::get()
        .uri(&format!("/api/items/{}", item_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
    println!("✅ E2E Item deletion verification test passed");
}

#[tokio::test]
async fn test_e2e_users_crud_workflow() {
    let factory = TestRepositoryFactory::new().await;

    let item_repository = factory.create_item_repository_for_service();
    let user_repository = factory.create_user_repository();

    let item_service = Arc::new(ItemService::new(item_repository));
    let user_service = Arc::new(UserService::new(user_repository));

    let app = test::init_service(create_test_app(item_service.clone(), user_service.clone())).await;

    // Test 1: Create a user
    let create_request = json!({
        "username": "e2e_test_user",
        "email": "e2e@test.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&create_request)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201);
    let created_user: Value = test::read_body_json(resp).await;
    let user_id = created_user["id"].as_u64().unwrap();

    assert_eq!(created_user["username"], "e2e_test_user");
    assert_eq!(created_user["email"], "e2e@test.com");
    println!("✅ E2E User creation test passed - ID: {}", user_id);

    // Test 2: Get the created user
    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let fetched_user: Value = test::read_body_json(resp).await;
    assert_eq!(fetched_user["id"], user_id);
    assert_eq!(fetched_user["username"], "e2e_test_user");
    println!("✅ E2E User retrieval test passed");

    // Test 3: Update the user
    let update_request = json!({
        "username": "updated_e2e_user",
        "email": "updated@test.com"
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/users/{}", user_id))
        .set_json(&update_request)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let updated_user: Value = test::read_body_json(resp).await;
    assert_eq!(updated_user["username"], "updated_e2e_user");
    assert_eq!(updated_user["email"], "updated@test.com");
    println!("✅ E2E User update test passed");

    // Test 4: Get all users (should contain our user)
    let req = test::TestRequest::get().uri("/api/users").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let users: Value = test::read_body_json(resp).await;
    let users_array = users.as_array().unwrap();
    assert!(!users_array.is_empty());

    let found_user = users_array.iter().find(|user| user["id"] == user_id);
    assert!(found_user.is_some());
    println!("✅ E2E Users list test passed");

    // Test 5: Delete the user
    let req = test::TestRequest::delete()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    println!("✅ E2E User deletion test passed");

    // Test 6: Verify user is deleted (should return 404)
    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
    println!("✅ E2E User deletion verification test passed");
}

#[tokio::test]
async fn test_e2e_error_handling() {
    let factory = TestRepositoryFactory::new().await;

    let item_repository = factory.create_item_repository_for_service();
    let user_repository = factory.create_user_repository();

    let item_service = Arc::new(ItemService::new(item_repository));
    let user_service = Arc::new(UserService::new(user_repository));

    let app = test::init_service(create_test_app(item_service.clone(), user_service.clone())).await;

    // Test 1: Get non-existent item
    let req = test::TestRequest::get()
        .uri("/api/items/99999")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
    println!("✅ E2E Item not found error handling test passed");

    // Test 2: Get non-existent user
    let req = test::TestRequest::get()
        .uri("/api/users/99999")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
    println!("✅ E2E User not found error handling test passed");

    // Test 3: Invalid JSON for item creation
    let req = test::TestRequest::post()
        .uri("/api/items")
        .set_payload("invalid json")
        .insert_header(("content-type", "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    println!("✅ E2E Invalid JSON error handling test passed");
}

#[tokio::test]
async fn test_e2e_multiple_items_workflow() {
    let factory = TestRepositoryFactory::new().await;

    let item_repository = factory.create_item_repository_for_service();
    let user_repository = factory.create_user_repository();

    let item_service = Arc::new(ItemService::new(item_repository));
    let user_service = Arc::new(UserService::new(user_repository));

    let app = test::init_service(create_test_app(item_service.clone(), user_service.clone())).await;

    // Create multiple items
    let mut item_ids = Vec::new();
    for i in 1..=3 {
        let create_request = json!({
            "name": format!("Item {}", i),
            "description": format!("Description {}", i)
        });

        let req = test::TestRequest::post()
            .uri("/api/items")
            .set_json(&create_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 201);
        let created_item: Value = test::read_body_json(resp).await;
        item_ids.push(created_item["id"].as_u64().unwrap());
    }

    // Get all items
    let req = test::TestRequest::get().uri("/api/items").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let items: Value = test::read_body_json(resp).await;
    let items_array = items.as_array().unwrap();
    assert_eq!(items_array.len(), 3);

    println!("✅ E2E Multiple items workflow test passed");
}

#[tokio::test]
async fn test_e2e_concurrent_operations() {
    let factory = TestRepositoryFactory::new().await;

    let item_repository = factory.create_item_repository_for_service();
    let user_repository = factory.create_user_repository();

    let item_service = Arc::new(ItemService::new(item_repository));
    let user_service = Arc::new(UserService::new(user_repository));

    let app = test::init_service(create_test_app(item_service.clone(), user_service.clone())).await;

    // Create multiple items concurrently
    let create_tasks: Vec<_> = (1..=5)
        .map(|i| {
            let app = &app;
            async move {
                let create_request = json!({
                    "name": format!("Concurrent Item {}", i),
                    "description": format!("Created concurrently {}", i)
                });

                let req = test::TestRequest::post()
                    .uri("/api/items")
                    .set_json(&create_request)
                    .to_request();
                test::call_service(app, req).await
            }
        })
        .collect();

    let responses = futures::future::join_all(create_tasks).await;

    // Verify all items were created successfully
    for (i, resp) in responses.iter().enumerate() {
        assert_eq!(resp.status(), 201);
        println!("✅ E2E Concurrent item {} creation test passed", i + 1);
    }

    // Verify all items exist
    let req = test::TestRequest::get().uri("/api/items").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let items: Value = test::read_body_json(resp).await;
    let items_array = items.as_array().unwrap();
    assert!(items_array.len() >= 5);

    println!(
        "✅ E2E Concurrent operations test passed - {} items created",
        items_array.len()
    );
}
