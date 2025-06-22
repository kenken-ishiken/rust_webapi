#!/bin/bash

# SLA Baseline Measurement Script
# This script helps establish performance baselines for the API

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASELINE_DIR="./results/baseline_$(date +%Y%m%d_%H%M%S)"
RESULTS_FILE="$BASELINE_DIR/baseline_report.md"

echo -e "${BLUE}=== SLA Baseline Measurement Tool ===${NC}"
echo ""

# Check if API is running
echo -e "${YELLOW}Checking API availability...${NC}"
API_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/health || echo "000")

if [ "$API_STATUS" != "200" ]; then
    echo -e "${RED}❌ API is not running or not accessible at http://localhost:8080${NC}"
    echo -e "${YELLOW}Please ensure the API server is running before measuring baselines.${NC}"
    echo ""
    echo "To start the API server:"
    echo "  1. Set up PostgreSQL: export DATABASE_URL=\"postgresql://username:password@localhost/rust_webapi\""
    echo "  2. Run migrations: sqlx migrate run"
    echo "  3. Start server: cargo run"
    exit 1
fi

echo -e "${GREEN}✅ API is accessible${NC}"
echo ""

# Create baseline directory
mkdir -p "$BASELINE_DIR"

# Initialize results file
cat > "$RESULTS_FILE" << EOF
# SLA Baseline Measurement Report

**Date**: $(date)
**API URL**: http://localhost:8080

## SLA Requirements
- 95th percentile response time < 250ms
- Error rate < 0.1%
- Throughput > 500 req/s with 1000 concurrent users

## Test Configuration
- Test Duration: 5 minutes
- Concurrent Users: 1000
- Scenario Mix:
  - 40% Product browsing (GET operations)
  - 25% Category navigation
  - 20% Item operations
  - 10% Mixed read operations
  - 5% Write operations

---

## Baseline Measurements

EOF

# Function to run a test and capture results
run_test() {
    local test_name=$1
    local test_type=$2
    local description=$3
    
    echo -e "${YELLOW}Running $test_name...${NC}"
    
    # Run the k6 test
    if ./run-tests.sh -t "$test_type" -o "$BASELINE_DIR" -d 5 > "$BASELINE_DIR/${test_type}_output.log" 2>&1; then
        echo -e "${GREEN}✅ $test_name completed${NC}"
        
        # Extract key metrics from the summary JSON
        SUMMARY_FILE=$(ls -t "$BASELINE_DIR"/*_summary.json | head -1)
        
        if [ -f "$SUMMARY_FILE" ]; then
            # Parse metrics using jq or python
            if command -v jq &> /dev/null; then
                P95=$(jq -r '.metrics.http_req_duration.values."p(95)"' "$SUMMARY_FILE" 2>/dev/null || echo "N/A")
                ERROR_RATE=$(jq -r '.metrics.http_req_failed.values.rate' "$SUMMARY_FILE" 2>/dev/null || echo "N/A")
                RPS=$(jq -r '.metrics.http_reqs.values.rate' "$SUMMARY_FILE" 2>/dev/null || echo "N/A")
            else
                # Fallback to grep if jq is not available
                P95=$(grep -o '"p(95)":[0-9.]*' "$SUMMARY_FILE" | cut -d: -f2 || echo "N/A")
                ERROR_RATE="Check JSON manually"
                RPS="Check JSON manually"
            fi
            
            # Append to results file
            cat >> "$RESULTS_FILE" << EOF
### $test_name

**Description**: $description

| Metric | Value | SLA Target | Status |
|--------|-------|-----------|--------|
| 95th percentile response time | ${P95}ms | < 250ms | $([ "$P95" != "N/A" ] && [ $(echo "$P95 < 250" | bc -l) -eq 1 ] && echo "✅ PASS" || echo "❌ FAIL") |
| Error rate | ${ERROR_RATE} | < 0.1% | TBD |
| Requests per second | ${RPS} | > 500 req/s | TBD |

**Log file**: ${test_type}_output.log

---

EOF
        else
            echo -e "${RED}❌ Could not find summary file for $test_name${NC}"
        fi
    else
        echo -e "${RED}❌ $test_name failed${NC}"
        cat >> "$RESULTS_FILE" << EOF
### $test_name

**Status**: ❌ FAILED

See log file for details: ${test_type}_output.log

---

EOF
    fi
    
    echo ""
}

# Run baseline tests
echo -e "${BLUE}Starting baseline measurements...${NC}"
echo ""

# 1. Smoke test (warm-up)
run_test "Smoke Test (Warm-up)" "smoke" "Basic functionality check and system warm-up"

# 2. SLA validation test
run_test "SLA Validation Test" "sla" "Full SLA compliance test with 1000 concurrent users"

# 3. Individual endpoint tests
run_test "Items API Test" "items" "Focused test on Items API endpoints"
run_test "Products API Test" "products" "Focused test on Products API endpoints"
run_test "Categories API Test" "categories" "Focused test on Categories API endpoints"

# Generate summary
echo -e "${BLUE}Generating summary...${NC}"

cat >> "$RESULTS_FILE" << EOF

## Summary and Recommendations

### Overall Status
$(grep -q "❌ FAIL" "$RESULTS_FILE" && echo "⚠️ **Some SLA targets were not met**" || echo "✅ **All SLA targets met**")

### Next Steps
1. Review detailed metrics in the individual test logs
2. Identify bottlenecks from failed SLA targets
3. Implement optimizations for problem areas
4. Re-run baseline measurements after optimizations

### Optimization Priorities
$(grep -B3 "❌ FAIL" "$RESULTS_FILE" | grep "^| " | head -5 || echo "No immediate optimizations needed")

---

**Report generated**: $(date)
EOF

# Display results
echo ""
echo -e "${GREEN}=== Baseline Measurement Complete ===${NC}"
echo -e "Results saved to: ${YELLOW}$BASELINE_DIR${NC}"
echo -e "Report: ${YELLOW}$RESULTS_FILE${NC}"
echo ""
echo -e "${BLUE}Quick Summary:${NC}"
tail -20 "$RESULTS_FILE" | grep -E "(PASS|FAIL|Status)" || echo "See full report for details"

# Open report if possible
if command -v xdg-open &> /dev/null; then
    echo ""
    echo -e "${YELLOW}Opening report in default editor...${NC}"
    xdg-open "$RESULTS_FILE" 2>/dev/null || true
fi