use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::application::service::item_service::ItemService;

// Include the generated proto code
tonic::include_proto!("item");

pub use item_service_server::{ItemService as ItemServiceTrait, ItemServiceServer};

pub struct ItemServiceImpl {
    service: Arc<ItemService>,
}

impl ItemServiceImpl {
    pub fn new(service: Arc<ItemService>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl ItemServiceTrait for ItemServiceImpl {
    async fn get_items(
        &self,
        _request: Request<GetItemsRequest>,
    ) -> Result<Response<GetItemsResponse>, Status> {
        match self.service.find_all().await {
            Ok(items) => {
                info!("gRPC: Fetched {} items", items.len());
                
                let grpc_items = items
                    .into_iter()
                    .map(|item| Item {
                        id: item.id,
                        name: item.name,
                        description: item.description,
                        deleted: item.deleted,
                        deleted_at: item.deleted_at.map(|dt| prost_types::Timestamp {
                            seconds: dt.timestamp(),
                            nanos: dt.timestamp_subsec_nanos() as i32,
                        }),
                    })
                    .collect();
                
                let response = GetItemsResponse {
                    items: grpc_items,
                };
                
                Ok(Response::new(response))
            }
            Err(e) => {
                info!("gRPC: Error fetching items: {}", e);
                Err(Status::internal(format!("アイテムの取得に失敗しました: {}", e)))
            }
        }
    }

    async fn get_item(
        &self,
        request: Request<GetItemRequest>,
    ) -> Result<Response<GetItemResponse>, Status> {
        let req = request.into_inner();
        
        match self.service.find_by_id(req.id).await {
            Ok(item) => {
                info!("gRPC: Fetched item {}", req.id);
                let grpc_item = Item {
                    id: item.id,
                    name: item.name,
                    description: item.description,
                    deleted: item.deleted,
                    deleted_at: item.deleted_at.map(|dt| prost_types::Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }),
                };
                
                let response = GetItemResponse {
                    item: Some(grpc_item),
                };
                
                Ok(Response::new(response))
            }
            Err(e) => {
                info!("gRPC: Item {} not found or error: {}", req.id, e);
                Err(Status::not_found(format!("アイテムが見つかりません: {}", e)))
            }
        }
    }

    async fn create_item(
        &self,
        request: Request<CreateItemRequest>,
    ) -> Result<Response<CreateItemResponse>, Status> {
        let req = request.into_inner();
        
        let create_request = crate::application::dto::item_dto::CreateItemRequest {
            name: req.name,
            description: req.description,
        };
        
        match self.service.create(create_request).await {
            Ok(new_item) => {
                info!("gRPC: Created item with id {}", new_item.id);
                
                let grpc_item = Item {
                    id: new_item.id,
                    name: new_item.name,
                    description: new_item.description,
                    deleted: new_item.deleted,
                    deleted_at: new_item.deleted_at.map(|dt| prost_types::Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }),
                };
                
                let response = CreateItemResponse {
                    item: Some(grpc_item),
                };
                
                Ok(Response::new(response))
            }
            Err(e) => {
                info!("gRPC: Error creating item: {}", e);
                Err(Status::internal(format!("アイテムの作成に失敗しました: {}", e)))
            }
        }
    }

    async fn update_item(
        &self,
        request: Request<UpdateItemRequest>,
    ) -> Result<Response<UpdateItemResponse>, Status> {
        let req = request.into_inner();
        
        let update_request = crate::application::dto::item_dto::UpdateItemRequest {
            name: req.name,
            description: req.description,
        };
        
        match self.service.update(req.id, update_request).await {
            Ok(updated_item) => {
                info!("gRPC: Updated item {}", req.id);
                let grpc_item = Item {
                    id: updated_item.id,
                    name: updated_item.name,
                    description: updated_item.description,
                    deleted: updated_item.deleted,
                    deleted_at: updated_item.deleted_at.map(|dt| prost_types::Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    }),
                };
                
                let response = UpdateItemResponse {
                    item: Some(grpc_item),
                };
                
                Ok(Response::new(response))
            }
            Err(e) => {
                info!("gRPC: Error updating item {}: {}", req.id, e);
                Err(Status::internal(format!("アイテムの更新に失敗しました: {}", e)))
            }
        }
    }

    async fn delete_item(
        &self,
        request: Request<DeleteItemRequest>,
    ) -> Result<Response<DeleteItemResponse>, Status> {
        let req = request.into_inner();
        
        match self.service.delete(req.id).await {
            Ok(_) => {
                info!("gRPC: Deleted item {}", req.id);
                let response = DeleteItemResponse { success: true };
                Ok(Response::new(response))
            }
            Err(e) => {
                info!("gRPC: Error deleting item {}: {}", req.id, e);
                Err(Status::internal(format!("アイテムの削除に失敗しました: {}", e)))
            }
        }
    }

    // Simplified implementations for the complex deletion methods
    async fn logical_delete_item(
        &self,
        request: Request<LogicalDeleteItemRequest>,
    ) -> Result<Response<LogicalDeleteItemResponse>, Status> {
        let req = request.into_inner();
        
        // For now, delegate to regular delete - this can be expanded based on actual service methods
        match self.service.delete(req.id).await {
            Ok(_) => {
                info!("gRPC: Logical delete item {}", req.id);
                let response = LogicalDeleteItemResponse { success: true };
                Ok(Response::new(response))
            }
            Err(e) => {
                info!("gRPC: Error logical deleting item {}: {}", req.id, e);
                Err(Status::internal(format!("論理削除に失敗しました: {}", e)))
            }
        }
    }

    async fn physical_delete_item(
        &self,
        _request: Request<PhysicalDeleteItemRequest>,
    ) -> Result<Response<PhysicalDeleteItemResponse>, Status> {
        // Physical delete not implemented yet
        Err(Status::unimplemented("物理削除は未実装です"))
    }

    async fn restore_item(
        &self,
        _request: Request<RestoreItemRequest>,
    ) -> Result<Response<RestoreItemResponse>, Status> {
        // Restore not implemented yet
        Err(Status::unimplemented("復元は未実装です"))
    }

    async fn validate_item_deletion(
        &self,
        _request: Request<ValidateItemDeletionRequest>,
    ) -> Result<Response<ValidateItemDeletionResponse>, Status> {
        // Validation not implemented yet
        Err(Status::unimplemented("削除検証は未実装です"))
    }

    async fn batch_delete_items(
        &self,
        _request: Request<BatchDeleteItemsRequest>,
    ) -> Result<Response<BatchDeleteItemsResponse>, Status> {
        // Batch delete not implemented yet
        Err(Status::unimplemented("バッチ削除は未実装です"))
    }

    async fn get_deleted_items(
        &self,
        _request: Request<GetDeletedItemsRequest>,
    ) -> Result<Response<GetDeletedItemsResponse>, Status> {
        // Get deleted items not implemented yet
        Err(Status::unimplemented("削除済みアイテム取得は未実装です"))
    }

    async fn get_item_deletion_log(
        &self,
        _request: Request<GetItemDeletionLogRequest>,
    ) -> Result<Response<GetItemDeletionLogResponse>, Status> {
        // Deletion log not implemented yet
        Err(Status::unimplemented("削除ログは未実装です"))
    }

    async fn get_deletion_logs(
        &self,
        _request: Request<GetDeletionLogsRequest>,
    ) -> Result<Response<GetDeletionLogsResponse>, Status> {
        // Deletion logs not implemented yet
        Err(Status::unimplemented("削除ログ一覧は未実装です"))
    }
}