import http from 'k6/http';
import { check } from 'k6';
import { KEYCLOAK_URL, KEYCLOAK_REALM, KEYCLOAK_CLIENT_ID, KEYCLOAK_CLIENT_SECRET } from '../config/constants.js';

// Get access token from Keycloak
export function getAccessToken(username, password) {
    const tokenUrl = `${KEYCLOAK_URL}/realms/${KEYCLOAK_REALM}/protocol/openid-connect/token`;
    
    const payload = {
        client_id: KEYCLOAK_CLIENT_ID,
        client_secret: KEYCLOAK_CLIENT_SECRET,
        username: username,
        password: password,
        grant_type: 'password'
    };

    const params = {
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        tags: { name: 'GetAccessToken' }
    };

    const response = http.post(tokenUrl, payload, params);
    
    check(response, {
        'authentication successful': (r) => r.status === 200,
        'access token present': (r) => JSON.parse(r.body).access_token !== undefined,
    });

    if (response.status !== 200) {
        console.error(`Authentication failed: ${response.status} - ${response.body}`);
        return null;
    }

    return JSON.parse(response.body).access_token;
}

// Create authenticated headers
export function getAuthHeaders(token) {
    return {
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
        }
    };
}

// Create headers with authentication and custom tags
export function getAuthHeadersWithTags(token, tags) {
    return {
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        tags: tags
    };
}