#!/bin/bash

# Test Environment Setup Script
# This script helps set up the environment for performance testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Test Environment Setup ===${NC}"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check service status
check_service() {
    local service=$1
    local port=$2
    
    if nc -z localhost "$port" 2>/dev/null; then
        echo -e "${GREEN}✅ $service is running on port $port${NC}"
        return 0
    else
        echo -e "${RED}❌ $service is not running on port $port${NC}"
        return 1
    fi
}

# 1. Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check for required tools
MISSING_TOOLS=()

if ! command_exists cargo; then
    MISSING_TOOLS+=("cargo (Rust)")
fi

if ! command_exists k6; then
    MISSING_TOOLS+=("k6")
fi

if ! command_exists jq; then
    MISSING_TOOLS+=("jq (optional but recommended)")
fi

if ! command_exists bc; then
    MISSING_TOOLS+=("bc (optional but recommended)")
fi

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo -e "${RED}Missing tools:${NC}"
    for tool in "${MISSING_TOOLS[@]}"; do
        echo "  - $tool"
    done
    echo ""
    echo "Please install missing tools before continuing."
    
    # Provide installation hints
    if [[ " ${MISSING_TOOLS[@]} " =~ " k6 " ]]; then
        echo ""
        echo "To install k6:"
        echo "  curl https://dl.k6.io/key.gpg | sudo apt-key add -"
        echo "  echo 'deb https://dl.k6.io/deb stable main' | sudo tee /etc/apt/sources.list.d/k6.list"
        echo "  sudo apt-get update && sudo apt-get install k6"
    fi
    
    if [[ " ${MISSING_TOOLS[@]} " =~ " jq " ]] || [[ " ${MISSING_TOOLS[@]} " =~ " bc " ]]; then
        echo ""
        echo "To install jq and bc:"
        echo "  sudo apt-get update && sudo apt-get install -y jq bc"
    fi
    
    exit 1
fi

echo -e "${GREEN}✅ All required tools are installed${NC}"
echo ""

# 2. Check services
echo -e "${YELLOW}Checking services...${NC}"

SERVICES_OK=true

# Check API server
if ! check_service "API Server" 8080; then
    SERVICES_OK=false
    echo ""
    echo "To start the API server:"
    echo "  1. Set environment variables:"
    echo "     export DATABASE_URL=\"postgresql://user:password@localhost/rust_webapi\""
    echo "     export JWT_SECRET=\"your-secret-key\""
    echo "     export RUST_LOG=\"info\""
    echo "  2. Run: cargo run"
fi

# Check PostgreSQL
if ! check_service "PostgreSQL" 5432; then
    SERVICES_OK=false
    echo ""
    echo "To start PostgreSQL:"
    echo "  sudo systemctl start postgresql"
    echo "  # Or using Docker:"
    echo "  docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15"
fi

# Check Keycloak (optional)
if ! check_service "Keycloak" 8081; then
    echo -e "${YELLOW}⚠️  Keycloak is not running (optional for basic tests)${NC}"
fi

echo ""

# 3. Create test data if API is running
if check_service "API Server" 8080 >/dev/null 2>&1; then
    echo -e "${YELLOW}Preparing test data...${NC}"
    
    # Create some test categories
    echo "Creating test categories..."
    for i in {1..5}; do
        curl -s -X POST http://localhost:8080/categories \
            -H "Content-Type: application/json" \
            -d "{\"name\": \"Test Category $i\", \"description\": \"Category for testing\"}" \
            >/dev/null 2>&1 || true
    done
    
    # Create some test products
    echo "Creating test products..."
    for i in {1..10}; do
        curl -s -X POST http://localhost:8080/products \
            -H "Content-Type: application/json" \
            -d "{\"name\": \"Test Product $i\", \"description\": \"Product for testing\", \"price\": $((RANDOM % 100 + 10)).99, \"category_id\": $((RANDOM % 5 + 1))}" \
            >/dev/null 2>&1 || true
    done
    
    # Create some test items
    echo "Creating test items..."
    for i in {1..20}; do
        curl -s -X POST http://localhost:8080/items \
            -H "Content-Type: application/json" \
            -d "{\"name\": \"Test Item $i\", \"description\": \"Item for testing\"}" \
            >/dev/null 2>&1 || true
    done
    
    echo -e "${GREEN}✅ Test data created${NC}"
else
    echo -e "${YELLOW}⚠️  Skipping test data creation (API not running)${NC}"
fi

echo ""

# 4. Summary
if [ "$SERVICES_OK" = true ]; then
    echo -e "${GREEN}=== Environment Ready ===${NC}"
    echo ""
    echo "You can now run performance tests:"
    echo "  cd k6"
    echo "  ./sla-baseline-measurement.sh"
else
    echo -e "${RED}=== Environment Not Ready ===${NC}"
    echo ""
    echo "Please start the required services before running tests."
fi

echo ""
echo -e "${BLUE}Environment Check Complete${NC}"

# Create a simple config file for k6 tests
if [ "$SERVICES_OK" = true ]; then
    cat > k6/.env << EOF
# Auto-generated k6 test configuration
API_BASE_URL=http://localhost:8080
KEYCLOAK_URL=http://localhost:8081
TEST_USERNAME=testuser
TEST_PASSWORD=testpass123
EOF
    echo -e "${GREEN}Created k6/.env configuration file${NC}"
fi