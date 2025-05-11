# Integration Testing with Testcontainers

This directory contains integration tests for the Rust WebAPI application, using testcontainers-rs to run isolated PostgreSQL instances for testing.

## Overview

The tests in this directory demonstrate how to use testcontainers-rs to run integration tests against real PostgreSQL databases without affecting your development or production environments.

## Key Components

### PostgreSQL Container Helper

The `helpers/postgres.rs` file provides a reusable PostgreSQL container setup that:

1. Creates an isolated PostgreSQL container for each test
2. Configures the database with proper credentials
3. Exposes connection information to your tests
4. Runs migrations
5. Cleans up containers automatically when tests are complete

### Test Files

- `db_tests.rs`: Basic tests to verify database connectivity
- `repository_tests.rs`: Tests for the PostgreSQL repository implementations

## Running the Tests

To run all integration tests:

```bash
cargo test
```

To run a specific test:

```bash
cargo test test_postgres_item_repository
```

## Best Practices

1. **Isolated Tests**: Each test gets its own container, ensuring test isolation
2. **Clean State**: Each test starts with a fresh database
3. **Real Behavior**: Tests run against the actual database type used in production
4. **Automatic Cleanup**: Containers are automatically stopped and removed when tests finish

## Implementation Notes

- The containers use dynamic port mapping to avoid port conflicts
- Tests run in parallel thanks to tokio's test runtime and isolated containers
- The PostgreSQL image is cached locally after the first run, making subsequent test runs faster
- No manual container cleanup or environment setup is needed