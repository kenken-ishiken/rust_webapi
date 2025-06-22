# SLA Baseline Measurement Guide

This guide explains how to set up and run SLA baseline measurements for the Rust WebAPI project.

## Prerequisites

### 1. System Requirements
- **k6 Load Testing Tool**: Install from https://k6.io/docs/getting-started/installation/
- **PostgreSQL**: Required for the API to function
- **jq**: Optional but recommended for JSON parsing (`sudo apt-get install jq`)
- **bc**: For numerical comparisons (`sudo apt-get install bc`)

### 2. API Server Setup

#### Option A: Using Docker Compose (Recommended)
```bash
# Start all services
docker-compose up -d

# Verify services are running
docker ps
```

#### Option B: Manual Setup
```bash
# 1. Install PostgreSQL
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib

# 2. Create database and user
sudo -u postgres psql
CREATE DATABASE rust_webapi;
CREATE USER webapi_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE rust_webapi TO webapi_user;
\q

# 3. Set environment variables
export DATABASE_URL="postgresql://webapi_user:your_password@localhost/rust_webapi"
export JWT_SECRET="your-secret-key"
export RUST_LOG="info"

# 4. Run database migrations
sqlx migrate run

# 5. Start the API server
cargo run
```

### 3. k6 Installation
```bash
# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# macOS
brew install k6

# Verify installation
k6 version
```

## Running Baseline Measurements

### Quick Start
```bash
cd k6
./sla-baseline-measurement.sh
```

### Manual Test Execution
If you prefer to run tests individually:

```bash
cd k6

# 1. Smoke test (warm-up)
./run-tests.sh -t smoke -o ./results/baseline

# 2. SLA validation test
./run-tests.sh -t sla -o ./results/baseline -d 5

# 3. Individual endpoint tests
./run-tests.sh -t items -o ./results/baseline
./run-tests.sh -t products -o ./results/baseline
./run-tests.sh -t categories -o ./results/baseline
```

## Understanding the Results

### SLA Targets
The system must meet these performance criteria:
- **95th percentile response time**: < 250ms
- **Error rate**: < 0.1%
- **Throughput**: > 500 req/s with 1000 concurrent users

### Test Scenarios
The SLA test simulates realistic user behavior:
- **40%** Product browsing (GET /products, GET /products/{id})
- **25%** Category navigation (GET /categories)
- **20%** Item operations (GET /items)
- **10%** Mixed read operations
- **5%** Write operations (POST/PUT/DELETE)

### Reading the Report
The baseline measurement generates a comprehensive report including:

1. **Summary Table**: Quick overview of pass/fail status
2. **Detailed Metrics**: For each test type
3. **Recommendations**: Based on failed targets
4. **Log Files**: Full k6 output for debugging

Example report structure:
```
results/baseline_YYYYMMDD_HHMMSS/
├── baseline_report.md          # Main report
├── smoke_output.log            # Smoke test logs
├── smoke_*.json               # Smoke test metrics
├── sla_output.log             # SLA test logs
├── sla_*.json                 # SLA test metrics
└── ...                        # Other test results
```

## Troubleshooting

### Common Issues

1. **API Not Running**
   ```
   ❌ API is not running or not accessible at http://localhost:8080
   ```
   **Solution**: Ensure the API server is running and accessible.

2. **k6 Not Found**
   ```
   Error: k6 is not installed
   ```
   **Solution**: Install k6 following the installation instructions above.

3. **Database Connection Failed**
   ```
   Error: Database connection error
   ```
   **Solution**: Verify PostgreSQL is running and DATABASE_URL is correct.

4. **Permission Denied**
   ```
   bash: ./sla-baseline-measurement.sh: Permission denied
   ```
   **Solution**: Make the script executable: `chmod +x sla-baseline-measurement.sh`

### Performance Issues

If SLA targets are not met:

1. **High Response Times**
   - Check database query performance
   - Review database indexes (see `migrations/20240101_add_performance_indexes.sql`)
   - Verify connection pool settings
   - Check CPU and memory usage during tests

2. **High Error Rate**
   - Check application logs for errors
   - Verify database connection stability
   - Review rate limiting settings
   - Check for resource exhaustion

3. **Low Throughput**
   - Increase worker threads (see `src/infrastructure/di/server.rs`)
   - Optimize database queries
   - Consider caching frequently accessed data
   - Review connection pool size

## Next Steps

After establishing baselines:

1. **Analyze Results**: Identify bottlenecks from failed SLA targets
2. **Implement Optimizations**: Based on the performance data
3. **Re-run Tests**: Verify improvements
4. **Document Changes**: Update baseline measurements

### Optimization Checklist
- [ ] Database query optimization
- [ ] Connection pool tuning
- [ ] HTTP server configuration
- [ ] Caching implementation
- [ ] Code-level optimizations
- [ ] Infrastructure scaling

## Advanced Configuration

### Custom Test Duration
```bash
# Run SLA test for 10 minutes
./run-tests.sh -t sla -o ./results/custom -d 10
```

### Custom User Count
```bash
# Run with 2000 concurrent users
./run-tests.sh -t sla -o ./results/custom -u 2000
```

### Environment-Specific Testing
```bash
# Test against staging environment
./run-tests.sh -t sla -e staging -o ./results/staging
```

## Continuous Performance Testing

For CI/CD integration:

```yaml
# Example GitHub Actions workflow
name: Performance Tests
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install k6
        run: |
          sudo gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6
      - name: Run SLA Tests
        run: |
          cd k6
          ./sla-baseline-measurement.sh
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: k6/results/
```

## References

- [k6 Documentation](https://k6.io/docs/)
- [Performance Optimization Guide](../../docs/performance-optimization.md)
- [API Documentation](../../docs/api-documentation.md)
- [Architecture Guide](../../docs/architecture-guide.md)