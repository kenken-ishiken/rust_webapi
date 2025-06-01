# gRPC API Documentation

This document describes the gRPC API endpoints that have been added alongside the existing REST API.

## Overview

The application now supports both REST and gRPC protocols:
- **REST API**: Available on `http://127.0.0.1:8080` (existing functionality)
- **gRPC API**: Available on `http://127.0.0.1:50051` (new functionality)

Both APIs share the same business logic and data access layer, ensuring consistency.

## Services

### User Service

**Proto file**: `proto/user.proto`

**Available methods**:
- `GetUsers()` - Get all users
- `GetUser(id)` - Get a user by ID
- `CreateUser(username, email)` - Create a new user
- `UpdateUser(id, username?, email?)` - Update an existing user
- `DeleteUser(id)` - Delete a user

### Item Service

**Proto file**: `proto/item.proto`

**Available methods**:
- `GetItems()` - Get all items
- `GetItem(id)` - Get an item by ID
- `CreateItem(name, description?)` - Create a new item
- `UpdateItem(id, name?, description?)` - Update an existing item
- `DeleteItem(id)` - Delete an item
- `LogicalDeleteItem(id)` - Logical delete (currently same as DeleteItem)

**Note**: Advanced deletion features (physical delete, restore, batch operations, etc.) are defined in the proto but not yet implemented. They return `UNIMPLEMENTED` status.

## Testing with grpcurl

You can test the gRPC API using `grpcurl` tool:

### Install grpcurl
```bash
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
```

### Example requests

**List all users:**
```bash
grpcurl -plaintext 127.0.0.1:50051 user.UserService/GetUsers
```

**Get a specific user:**
```bash
grpcurl -plaintext -d '{"id": 1}' 127.0.0.1:50051 user.UserService/GetUser
```

**Create a user:**
```bash
grpcurl -plaintext -d '{"username": "testuser", "email": "test@example.com"}' 127.0.0.1:50051 user.UserService/CreateUser
```

**List all items:**
```bash
grpcurl -plaintext 127.0.0.1:50051 item.ItemService/GetItems
```

**Create an item:**
```bash
grpcurl -plaintext -d '{"name": "Test Item", "description": "Test Description"}' 127.0.0.1:50051 item.ItemService/CreateItem
```

## Using with gRPC Clients

The proto files can be used to generate client code for various languages:

- **Rust**: tonic
- **Go**: protoc-gen-go + protoc-gen-go-grpc
- **Python**: grpcio-tools
- **Java**: protobuf-gradle-plugin
- **JavaScript/TypeScript**: grpc-web

## Server Reflection

Server reflection is not currently enabled. To enable it, add the reflection service to the gRPC server in `main.rs`.

## Architecture

The gRPC implementation follows the same layered architecture as the REST API:

```
gRPC Request -> gRPC Handler -> Service Layer -> Repository Layer -> Database
```

This ensures that business logic remains consistent between REST and gRPC interfaces.