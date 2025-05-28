#!/bin/bash

# k6 Load Testing Runner Script
# This script helps run different types of load tests

set -e

# Default values
TEST_TYPE="smoke"
OUTPUT_DIR="./results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --test TYPE      Test type: smoke, load, stress, spike, items, products, categories, users (default: smoke)"
    echo "  -o, --output DIR     Output directory for results (default: ./results)"
    echo "  -e, --env ENV        Environment: local, dev, staging, prod (default: local)"
    echo "  -d, --duration MIN   Override test duration in minutes (optional)"
    echo "  -u, --users NUM      Override max users (optional)"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  API_BASE_URL         API base URL (default: http://localhost:8080)"
    echo "  KEYCLOAK_URL         Keycloak URL (default: http://localhost:8081)"
    echo "  KEYCLOAK_CLIENT_SECRET  Keycloak client secret (required for auth)"
    echo "  TEST_USERNAME        Test username (default: testuser)"
    echo "  TEST_PASSWORD        Test password (default: testpass123)"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--test)
            TEST_TYPE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -e|--env)
            ENV="$2"
            shift 2
            ;;
        -d|--duration)
            DURATION="$2"
            shift 2
            ;;
        -u|--users)
            MAX_USERS="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Set environment-specific variables
case ${ENV:-local} in
    local)
        export API_BASE_URL=${API_BASE_URL:-"http://localhost:8080"}
        export KEYCLOAK_URL=${KEYCLOAK_URL:-"http://localhost:8081"}
        ;;
    dev)
        export API_BASE_URL=${API_BASE_URL:-"https://api-dev.example.com"}
        export KEYCLOAK_URL=${KEYCLOAK_URL:-"https://auth-dev.example.com"}
        ;;
    staging)
        export API_BASE_URL=${API_BASE_URL:-"https://api-staging.example.com"}
        export KEYCLOAK_URL=${KEYCLOAK_URL:-"https://auth-staging.example.com"}
        ;;
    prod)
        export API_BASE_URL=${API_BASE_URL:-"https://api.example.com"}
        export KEYCLOAK_URL=${KEYCLOAK_URL:-"https://auth.example.com"}
        ;;
esac

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}Error: k6 is not installed${NC}"
    echo "Please install k6 from https://k6.io/docs/getting-started/installation/"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Determine test file
case $TEST_TYPE in
    smoke)
        TEST_FILE="tests/smoke/smoke-test.js"
        TEST_NAME="Smoke Test"
        ;;
    load)
        TEST_FILE="tests/load/load-test.js"
        TEST_NAME="Load Test"
        ;;
    stress)
        TEST_FILE="tests/stress/stress-test.js"
        TEST_NAME="Stress Test"
        ;;
    spike)
        TEST_FILE="tests/spike/spike-test.js"
        TEST_NAME="Spike Test"
        ;;
    items)
        TEST_FILE="tests/items.js"
        TEST_NAME="Items API Test"
        ;;
    products)
        TEST_FILE="tests/products.js"
        TEST_NAME="Products API Test"
        ;;
    categories)
        TEST_FILE="tests/categories.js"
        TEST_NAME="Categories API Test"
        ;;
    users)
        TEST_FILE="tests/users.js"
        TEST_NAME="Users API Test"
        ;;
    *)
        echo -e "${RED}Error: Unknown test type: $TEST_TYPE${NC}"
        usage
        ;;
esac

# Check if test file exists
if [ ! -f "$TEST_FILE" ]; then
    echo -e "${RED}Error: Test file not found: $TEST_FILE${NC}"
    exit 1
fi

# Build k6 command
K6_CMD="k6 run"

# Add output options
K6_CMD="$K6_CMD --out json=$OUTPUT_DIR/${TEST_TYPE}_${TIMESTAMP}.json"
K6_CMD="$K6_CMD --summary-export=$OUTPUT_DIR/${TEST_TYPE}_${TIMESTAMP}_summary.json"

# Add optional overrides
if [ -n "$DURATION" ]; then
    K6_CMD="$K6_CMD --duration ${DURATION}m"
fi

if [ -n "$MAX_USERS" ]; then
    K6_CMD="$K6_CMD --vus $MAX_USERS"
fi

# Add test file
K6_CMD="$K6_CMD $TEST_FILE"

# Display test information
echo -e "${GREEN}=== k6 Load Testing ===${NC}"
echo -e "Test Type: ${YELLOW}$TEST_NAME${NC}"
echo -e "Environment: ${YELLOW}${ENV:-local}${NC}"
echo -e "API URL: ${YELLOW}$API_BASE_URL${NC}"
echo -e "Output Dir: ${YELLOW}$OUTPUT_DIR${NC}"
echo ""
echo -e "${GREEN}Starting test...${NC}"
echo ""

# Run the test
if $K6_CMD; then
    echo ""
    echo -e "${GREEN}✅ Test completed successfully!${NC}"
    echo -e "Results saved to: ${YELLOW}$OUTPUT_DIR${NC}"
    
    # Generate HTML report if k6-reporter is available
    if command -v k6-reporter &> /dev/null; then
        echo -e "${GREEN}Generating HTML report...${NC}"
        k6-reporter "$OUTPUT_DIR/${TEST_TYPE}_${TIMESTAMP}_summary.json" \
            --out "$OUTPUT_DIR/${TEST_TYPE}_${TIMESTAMP}_report.html"
        echo -e "HTML report: ${YELLOW}$OUTPUT_DIR/${TEST_TYPE}_${TIMESTAMP}_report.html${NC}"
    fi
else
    echo ""
    echo -e "${RED}❌ Test failed!${NC}"
    exit 1
fi