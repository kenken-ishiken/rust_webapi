# Performance Optimization Guide

## SLA Requirements (from REMAINING_IMPROVEMENTS.md)

The system must meet the following Service Level Agreement (SLA) criteria:

- **95th percentile response time**: < 250ms
- **Error rate**: < 0.1%
- **Throughput**: > 500 req/s with 1000 concurrent users
- **Availability**: 99.9% uptime

## Performance Testing

### Running SLA Validation Test

```bash
cd k6
./run-tests.sh -t sla
```

This test will:
1. Ramp up to 1000 concurrent users over 2 minutes
2. Maintain 1000 users for 5 minutes
3. Validate all SLA thresholds
4. Generate detailed performance metrics

### Test Scenarios

The SLA validation test simulates realistic user behavior with:
- **40%** - Product browsing (read-heavy)
- **25%** - Category navigation
- **20%** - Item operations
- **10%** - Mixed read operations
- **5%** - Write operations

## Performance Optimization Strategies

### 1. Database Optimizations

#### Connection Pooling
```rust
// Current configuration in src/infrastructure/config/mod.rs
pub struct DatabaseConfig {
    pub max_connections: u32,     // Recommend: 100-200 for production
    pub min_connections: u32,      // Recommend: 10-20
    pub connect_timeout: u64,      // Recommend: 5 seconds
    pub idle_timeout: Option<u64>, // Recommend: 10 minutes
}
```

#### Query Optimization
- **Use indexes** on frequently queried columns:
  ```sql
  CREATE INDEX idx_products_category_id ON products(category_id);
  CREATE INDEX idx_products_is_active ON products(is_active);
  CREATE INDEX idx_items_is_deleted ON items(is_deleted);
  ```

- **Optimize pagination** with cursor-based pagination for large datasets:
  ```rust
  // Instead of OFFSET/LIMIT
  SELECT * FROM products WHERE id > $1 ORDER BY id LIMIT $2;
  ```

### 2. Caching Strategy

#### In-Memory Caching
```rust
use moka::future::Cache;

// Add to AppContainer
pub struct AppContainer {
    cache: Cache<String, Vec<u8>>, // TTL-based cache
    // ... existing fields
}

// Cache configuration
let cache = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300)) // 5 minutes
    .build();
```

#### Cache Layers
1. **Application-level cache**: Frequently accessed entities
2. **Database query cache**: Complex query results
3. **HTTP response cache**: Static responses

### 3. Async Optimization

#### Concurrent Request Handling
```rust
// Use tokio::spawn for parallel operations
let futures = vec![
    tokio::spawn(fetch_products()),
    tokio::spawn(fetch_categories()),
    tokio::spawn(fetch_items()),
];

let results = futures::future::join_all(futures).await;
```

#### Batch Processing
```rust
// Process items in batches
const BATCH_SIZE: usize = 100;

for chunk in items.chunks(BATCH_SIZE) {
    process_batch(chunk).await?;
}
```

### 4. HTTP Server Optimization

#### Actix Web Configuration
```rust
HttpServer::new(move || {
    App::new()
        .app_data(web::JsonConfig::default()
            .limit(4096) // Limit request size
            .error_handler(|err, _req| {
                AppError::bad_request(err.to_string()).into()
            }))
        .wrap(middleware::Compress::default()) // Enable compression
        .wrap(middleware::NormalizePath::trim()) // Normalize paths
})
.workers(num_cpus::get() * 2) // Optimize worker count
.keep_alive(Duration::from_secs(75)) // Keep-alive timeout
.client_timeout(Duration::from_secs(60)) // Client timeout
.bind(("0.0.0.0", 8080))?
.run()
.await
```

### 5. Code-Level Optimizations

#### Avoid Unnecessary Allocations
```rust
// Use &str instead of String when possible
pub fn find_by_name(&self, name: &str) -> Result<Option<Item>, AppError>

// Use Cow for conditional ownership
use std::borrow::Cow;
pub fn process_name(name: Cow<'_, str>) -> String
```

#### Optimize Serialization
```rust
// Use serde's derive features efficiently
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}
```

### 6. Monitoring and Observability

#### Metrics Collection
- Response time percentiles (p50, p95, p99)
- Request rate and error rate
- Database connection pool usage
- Memory and CPU utilization

#### Performance Dashboards
1. **Prometheus + Grafana** for metrics visualization
2. **Jaeger** for distributed tracing
3. **ELK Stack** for log aggregation

## Performance Baseline

Based on current implementation (as of December 2024):

| Metric | Current | Target | Status |
|--------|---------|--------|---------|
| P95 Response Time | TBD | < 250ms | 🔄 Testing |
| P99 Response Time | TBD | < 400ms | 🔄 Testing |
| Error Rate | TBD | < 0.1% | 🔄 Testing |
| Throughput (1000 users) | TBD | > 500 req/s | 🔄 Testing |
| Memory Usage | TBD | < 1GB | 🔄 Testing |
| CPU Usage | TBD | < 80% | 🔄 Testing |

## Optimization Checklist

- [ ] Enable database connection pooling with optimal settings
- [ ] Add indexes on frequently queried columns
- [ ] Implement application-level caching
- [ ] Enable HTTP response compression
- [ ] Optimize JSON serialization
- [ ] Configure optimal worker thread count
- [ ] Implement request rate limiting
- [ ] Add circuit breakers for external services
- [ ] Enable query result caching
- [ ] Optimize container resource limits

## Next Steps

1. **Run baseline performance tests**:
   ```bash
   cd k6
   ./run-tests.sh -t sla -o ./results/baseline
   ```

2. **Analyze bottlenecks** using profiling tools:
   ```bash
   cargo flamegraph --bin rust-webapi
   ```

3. **Implement optimizations** based on findings

4. **Re-run tests** to validate improvements

5. **Document results** and update baseline metrics

## References

- [Actix Web Performance Guide](https://actix.rs/docs/server/)
- [Tokio Performance Tuning](https://tokio.rs/tokio/tutorial)
- [PostgreSQL Performance Tips](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [k6 Performance Testing Best Practices](https://k6.io/docs/testing-guides/)