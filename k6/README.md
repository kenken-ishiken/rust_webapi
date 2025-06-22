# k6 Load Testing Suite for Rust WebAPI

This directory contains comprehensive load testing scripts for the Rust WebAPI using [k6](https://k6.io/).

## Overview

The test suite includes:
- **Endpoint-specific tests**: Detailed tests for Items, Products, Categories, and Users APIs
- **Performance scenarios**: Smoke, Load, Stress, and Spike tests
- **Utilities**: Authentication helpers, data generators, and configuration

## Prerequisites

1. **Install k6**:
   ```bash
   # macOS
   brew install k6
   
   # Linux
   sudo apt-get install k6
   
   # Windows
   choco install k6
   ```

2. **Environment Setup**:
   - Ensure the Rust WebAPI is running locally or accessible
   - Keycloak authentication service must be running
   - Set up test user credentials in Keycloak

## Quick Start

1. **Set up test environment**:
   ```bash
   ./setup-test-environment.sh
   ```

2. **Run SLA baseline measurements**:
   ```bash
   ./sla-baseline-measurement.sh
   ```

3. **Run individual tests**:
   ```bash
   ./run-tests.sh -t smoke      # Minimal load test
   ./run-tests.sh -t load       # Normal expected load
   ./run-tests.sh -t sla        # SLA validation test
   ./run-tests.sh -t products   # Test product endpoints
   ```

## Test Types

### Performance Scenarios

1. **Smoke Test** (`tests/smoke/smoke-test.js`)
   - Duration: ~2 minutes
   - Max VUs: 5
   - Purpose: Basic functionality verification
   - Thresholds: p95 < 1s, error rate < 5%

2. **Load Test** (`tests/load/load-test.js`)
   - Duration: ~16 minutes
   - Max VUs: 100
   - Purpose: Normal expected load
   - Thresholds: p95 < 500ms, error rate < 1%

3. **Stress Test** (`tests/stress/stress-test.js`)
   - Duration: ~26 minutes
   - Max VUs: 300
   - Purpose: Find breaking point
   - Thresholds: p95 < 2s, error rate < 10%

4. **Spike Test** (`tests/spike/spike-test.js`)
   - Duration: ~5 minutes
   - Spike: 50 → 500 VUs
   - Purpose: Test sudden traffic increase
   - Thresholds: p95 < 3s, error rate < 20%

5. **SLA Validation Test** (`tests/sla/sla-validation-test.js`)
   - Duration: 5 minutes (configurable)
   - VUs: 1000 concurrent users
   - Purpose: Validate against SLA requirements
   - SLA Targets:
     - 95th percentile response time < 250ms
     - Error rate < 0.1%
     - Throughput > 500 req/s
   - Scenario Mix:
     - 40% Product browsing
     - 25% Category navigation
     - 20% Item operations
     - 10% Mixed reads
     - 5% Write operations

### API Endpoint Tests

1. **Items API** (`tests/items.js`)
   - CRUD operations
   - Batch operations
   - Deletion features (logical/physical)
   - Deletion logs

2. **Products API** (`tests/products.js`)
   - CRUD operations
   - Search and filtering
   - Price/inventory updates
   - Image management
   - Batch operations
   - Reports (low stock, out of stock)

3. **Categories API** (`tests/categories.js`)
   - CRUD operations
   - Hierarchical operations
   - Tree navigation
   - Move operations

4. **Users API** (`tests/users.js`)
   - CRUD operations
   - Authentication tests
   - Bulk operations

## Configuration

### Environment Variables

```bash
# API Configuration
export API_BASE_URL="http://localhost:8080"

# Keycloak Configuration
export KEYCLOAK_URL="http://localhost:8081"
export KEYCLOAK_REALM="rust-webapi"
export KEYCLOAK_CLIENT_ID="api-client"
export KEYCLOAK_CLIENT_SECRET="your-client-secret"

# Test User Credentials
export TEST_USERNAME="testuser"
export TEST_PASSWORD="testpass123"
```

### Test Stages

Edit `config/constants.js` to customize test stages:

```javascript
// Example: Custom load test stages
export const LOAD_TEST_STAGES = [
    { duration: '2m', target: 50 },   // Ramp up
    { duration: '5m', target: 50 },   // Steady state
    { duration: '2m', target: 0 },    // Ramp down
];
```

## Running Tests

### Using the Runner Script

```bash
# Basic usage
./run-tests.sh -t <test-type>

# With custom output directory
./run-tests.sh -t load -o ./my-results

# For different environments
./run-tests.sh -t stress -e staging

# Override duration and users
./run-tests.sh -t load -d 10 -u 200
```

### Direct k6 Commands

```bash
# Run specific test
k6 run tests/products.js

# With custom options
k6 run --vus 50 --duration 5m tests/load/load-test.js

# Output results to file
k6 run --out json=results.json tests/stress/stress-test.js

# With HTML report
k6 run --out web-dashboard tests/spike/spike-test.js
```

## SLA Baseline Measurements

### Running Baseline Tests

The `sla-baseline-measurement.sh` script automates the process of establishing performance baselines:

```bash
# Run full baseline measurement suite
./sla-baseline-measurement.sh
```

This script will:
1. Check API availability
2. Run smoke test (warm-up)
3. Run SLA validation test
4. Run individual endpoint tests
5. Generate a comprehensive report

### Understanding Baseline Reports

Reports are saved to `results/baseline_<timestamp>/`:
- `baseline_report.md`: Summary report with pass/fail status
- Individual test logs and metrics
- SLA compliance summary

### Establishing Baselines

1. **Initial Baseline**: Run tests on a clean deployment
2. **Regular Updates**: Re-run after significant changes
3. **Compare Results**: Track performance trends over time
4. **Document Changes**: Note configuration or code changes

## Analyzing Results

### Console Output
k6 provides real-time metrics during test execution:
- **http_req_duration**: Response time percentiles
- **http_req_failed**: Error rate
- **http_reqs**: Requests per second
- **vus**: Active virtual users

### JSON Output
Results are saved to `results/` directory:
- `<test>_<timestamp>.json`: Detailed metrics
- `<test>_<timestamp>_summary.json`: Summary statistics

### Key Metrics to Monitor

1. **Response Times**
   - p95: 95th percentile (95% of requests are faster)
   - p99: 99th percentile (99% of requests are faster)

2. **Error Rate**
   - Should be < 1% for normal operations
   - May increase during stress/spike tests

3. **Throughput**
   - Requests per second (RPS)
   - Should remain stable under load

4. **Resource Utilization**
   - Monitor API server CPU/memory
   - Database connection pool usage
   - Network I/O

## Best Practices

1. **Start Small**: Always run smoke tests first
2. **Gradual Increase**: Progress from smoke → load → stress
3. **Monitor Infrastructure**: Watch server resources during tests
4. **Baseline First**: Establish performance baselines
5. **Regular Testing**: Run tests after significant changes
6. **Clean Data**: Tests create and clean up test data automatically

## Troubleshooting

### Authentication Failures
```bash
# Check Keycloak is running
curl http://localhost:8081/health

# Verify credentials
export KEYCLOAK_CLIENT_SECRET="correct-secret"
```

### Connection Errors
```bash
# Check API is running
curl http://localhost:8080/api/health

# Check network access
nc -zv localhost 8080
```

### High Error Rates
- Check server logs for errors
- Verify database connections
- Monitor system resources
- Reduce load if necessary

## CI/CD Integration

### GitHub Actions Example
```yaml
- name: Run Load Tests
  run: |
    cd k6
    ./run-tests.sh -t smoke -e staging
    ./run-tests.sh -t load -e staging
```

### Jenkins Example
```groovy
stage('Load Test') {
    steps {
        sh 'cd k6 && ./run-tests.sh -t load -o ${WORKSPACE}/test-results'
    }
}
```

## Contributing

When adding new tests:
1. Follow existing patterns in test files
2. Use appropriate tags for metrics grouping
3. Include proper error handling
4. Add cleanup for created resources
5. Update this README with new test information

## Resources

- [k6 Documentation](https://k6.io/docs/)
- [k6 API Reference](https://k6.io/docs/javascript-api/)
- [Performance Testing Best Practices](https://k6.io/docs/testing-guides/)