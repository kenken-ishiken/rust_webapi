#!/bin/bash

# Keycloak setup script for testing
set -e

KEYCLOAK_URL="http://localhost:8081"
ADMIN_USER="admin"
ADMIN_PASSWORD="admin"
REALM_NAME="rust-webapi"
CLIENT_ID="api-client"
CLIENT_SECRET="test-client-secret"
TEST_USER="testuser"
TEST_PASSWORD="testpass123"

echo "Setting up Keycloak for testing..."

# Wait a bit for Keycloak to be ready
echo "Waiting for Keycloak to be ready..."
sleep 5

# Get admin token
echo "Getting admin token..."
ADMIN_TOKEN=$(curl -s -X POST "${KEYCLOAK_URL}/realms/master/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=${ADMIN_USER}" \
    -d "password=${ADMIN_PASSWORD}" \
    -d "grant_type=password" \
    -d "client_id=admin-cli" | jq -r '.access_token')

if [ -z "$ADMIN_TOKEN" ] || [ "$ADMIN_TOKEN" = "null" ]; then
    echo "❌ Failed to get admin token"
    exit 1
fi
echo "✅ Admin token obtained"

# Create realm
echo "Creating realm '${REALM_NAME}'..."
curl -s -X POST "${KEYCLOAK_URL}/admin/realms" \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "realm": "'${REALM_NAME}'",
        "enabled": true,
        "registrationAllowed": false,
        "loginWithEmailAllowed": true,
        "duplicateEmailsAllowed": false,
        "resetPasswordAllowed": true,
        "editUsernameAllowed": false,
        "bruteForceProtected": true
    }' || echo "Realm might already exist"

# Create client
echo "Creating client '${CLIENT_ID}'..."
curl -s -X POST "${KEYCLOAK_URL}/admin/realms/${REALM_NAME}/clients" \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "clientId": "'${CLIENT_ID}'",
        "secret": "'${CLIENT_SECRET}'",
        "enabled": true,
        "directAccessGrantsEnabled": true,
        "publicClient": false,
        "protocol": "openid-connect",
        "standardFlowEnabled": true,
        "implicitFlowEnabled": false,
        "serviceAccountsEnabled": true,
        "authorizationServicesEnabled": false
    }' || echo "Client might already exist"

# Create test user
echo "Creating test user '${TEST_USER}'..."
curl -s -X POST "${KEYCLOAK_URL}/admin/realms/${REALM_NAME}/users" \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "'${TEST_USER}'",
        "email": "'${TEST_USER}'@example.com",
        "firstName": "Test",
        "lastName": "User",
        "enabled": true,
        "emailVerified": true,
        "credentials": [{
            "type": "password",
            "value": "'${TEST_PASSWORD}'",
            "temporary": false
        }]
    }' || echo "User might already exist"

echo ""
echo "✅ Keycloak setup complete!"
echo ""
echo "Configuration:"
echo "  Keycloak URL: ${KEYCLOAK_URL}"
echo "  Realm: ${REALM_NAME}"
echo "  Client ID: ${CLIENT_ID}"
echo "  Client Secret: ${CLIENT_SECRET}"
echo "  Test User: ${TEST_USER}"
echo "  Test Password: ${TEST_PASSWORD}"
echo ""
echo "Admin Console: ${KEYCLOAK_URL}/admin"
echo "  Username: ${ADMIN_USER}"
echo "  Password: ${ADMIN_PASSWORD}" 