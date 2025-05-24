# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Run

```bash
# Build the project
cargo build

# Run locally
cargo run

# Run with Docker Compose (includes PostgreSQL)
docker-compose up -d
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Generate test coverage report (requires llvm-tools-preview)
./scripts/coverage.sh
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets -- -D warnings
```

### Database

The project uses PostgreSQL with sqlx. The database schema is defined in:
- `initdb/01_create_tables.sql`

## Documentation

Detailed documentation is available in the `docs/` directory:
- `docs/api-reference.md`: API endpoint specifications
- `docs/architecture-guide.md`: System architecture and design
- `docs/development-guide.md`: Development workflow and tools
- `docs/operations-guide.md`: Deployment and operations

## Architecture

This project follows Domain-Driven Design principles with a layered architecture:

1. **Domain Layer** (`crates/domain/` and `src/app_domain/`)
   - Business entities and value objects
   - Repository interfaces (traits)
   - Pure business logic

2. **Application Layer** (`src/application/`)
   - DTOs (Data Transfer Objects) for API communication
   - Services implementing use cases using domain objects

3. **Infrastructure Layer** (`src/infrastructure/`)
   - Repository implementations using PostgreSQL
   - Authentication with Keycloak
   - Logging and metrics using tracing and Prometheus (slog utilities remain for reference)
   - Configuration management

4. **Presentation Layer** (`src/presentation/`)
   - HTTP handlers using actix-web
   - Request/response serialization
   - Route definitions

## Key Components

### Repositories

Repository interfaces are defined in the domain layer, while implementations are in the infrastructure layer:
- `domain::repository::item_repository::ItemRepositoryImpl`
- `infrastructure::repository::item_repository::PostgresItemRepository`

### Services

Services implement application use cases by orchestrating domain operations:
- `application::service::item_service::ItemService`
- `application::service::user_service::UserService`

### API Handlers

HTTP handlers process requests, call services, and format responses:
- `presentation::api::item_handler::ItemHandler`
- `presentation::api::user_handler::UserHandler`

### Authentication

JWT authentication using Keycloak:
- `infrastructure::auth::keycloak::KeycloakAuth`
- `infrastructure::auth::middleware`

### Observability

The project implements comprehensive observability via:
- **Logging**: Structured JSON logging with `tracing` (slog utilities kept for reference)
- **Metrics**: Prometheus metrics exposed at `/api/metrics`
- **Tracing**: OpenTelemetry integration for distributed tracing

## Environment Setup

The application uses environment variables for configuration, typically loaded from a `.env` file:

```
DATABASE_URL=postgres://postgres:password@postgres:5432/rustwebapi
KEYCLOAK_AUTH_SERVER_URL=http://localhost:8081
KEYCLOAK_REALM=rust-webapi
KEYCLOAK_CLIENT_ID=api-client
```

## Kubernetes Deployment

The application can be deployed to Kubernetes:

```bash
# Deploy with kustomize
kubectl apply -k k8s/base

# Deploy to a specific environment
kubectl apply -k k8s/overlays/dev
```

See `k8s/README.md` for detailed deployment instructions.