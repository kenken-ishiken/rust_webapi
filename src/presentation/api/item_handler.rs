use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::application::dto::item_dto::{CreateItemRequest, UpdateItemRequest};
use crate::application::service::item_service::ItemService;

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

    pub async fn get_items(data: web::Data<ItemHandler>) -> impl Responder {
        let items = data.service.find_all().await;
        HttpResponse::Ok().json(items)
    }

    pub async fn get_item(
        data: web::Data<ItemHandler>, 
        path: web::Path<u64>,
    ) -> impl Responder {
        let item_id = path.into_inner();
        match data.service.find_by_id(item_id).await {
            Some(item) => HttpResponse::Ok().json(item),
            None => HttpResponse::NotFound().json("アイテムが見つかりません"),
        }
    }

    pub async fn create_item(
        data: web::Data<ItemHandler>,
        item: web::Json<CreateItemRequest>,
    ) -> impl Responder {
        let new_item = data.service.create(item.into_inner()).await;
        HttpResponse::Created().json(new_item)
    }

    pub async fn update_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
        item: web::Json<UpdateItemRequest>,
    ) -> impl Responder {
        let item_id = path.into_inner();
        match data.service.update(item_id, item.into_inner()).await {
            Some(updated_item) => HttpResponse::Ok().json(updated_item),
            None => HttpResponse::NotFound().json("アイテムが見つかりません"),
        }
    }

    pub async fn delete_item(
        data: web::Data<ItemHandler>,
        path: web::Path<u64>,
    ) -> impl Responder {
        let item_id = path.into_inner();
        if data.service.delete(item_id).await {
            HttpResponse::Ok().json("アイテムを削除しました")
        } else {
            HttpResponse::NotFound().json("アイテムが見つかりません")
        }
    }
}