import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { API_BASE_URL, SMOKE_TEST_STAGES, DEFAULT_THRESHOLDS } from '../../config/constants.js';
import { getAccessToken, getAuthHeaders } from '../../utils/auth.js';
import { checkResponse, sleepWithJitter } from '../../utils/helpers.js';

// Smoke test - minimal load to verify system works
export let options = {
    stages: SMOKE_TEST_STAGES,
    thresholds: {
        ...DEFAULT_THRESHOLDS,
        'http_req_duration': ['p(95)<1000'], // Relaxed for smoke test
        'http_req_failed': ['rate<0.05'],    // Allow up to 5% failure rate
    },
};

export function setup() {
    const token = getAccessToken('testuser', 'testpass123');
    
    if (!token) {
        throw new Error('Failed to authenticate - smoke test cannot proceed');
    }
    
    console.log('✅ Smoke test setup completed successfully');
    return { token };
}

export default function (data) {
    const token = data.token;
    const authHeaders = getAuthHeaders(token);
    
    group('Smoke Test - Basic Health Checks', () => {
        // Health check (no auth required)
        group('Health Check', () => {
            const response = http.get(`${API_BASE_URL}/`);
            checkResponse(response, 200, 'Root health check');
        });
        
        sleep(sleepWithJitter(0.5));
        
        // API health check
        group('API Health', () => {
            const response = http.get(`${API_BASE_URL}/api/health`);
            checkResponse(response, 200, 'API health check');
        });
        
        sleep(sleepWithJitter(0.5));
        
        // Metrics endpoint
        group('Metrics', () => {
            const response = http.get(`${API_BASE_URL}/api/metrics`);
            check(response, {
                'Metrics endpoint accessible': (r) => r.status === 200,
                'Metrics contain data': (r) => r.body.includes('api_request_duration_seconds')
            });
        });
    });
    
    sleep(sleepWithJitter(1));
    
    group('Smoke Test - Core Endpoints', () => {
        // Test each major endpoint with minimal operations
        
        // Items endpoint
        group('Items Endpoint', () => {
            const response = http.get(`${API_BASE_URL}/api/items`, authHeaders);
            checkResponse(response, 200, 'Get items');
        });
        
        sleep(sleepWithJitter(0.5));
        
        // Users endpoint
        group('Users Endpoint', () => {
            const response = http.get(`${API_BASE_URL}/api/users`, authHeaders);
            checkResponse(response, 200, 'Get users');
        });
        
        sleep(sleepWithJitter(0.5));
        
        // Categories endpoint
        group('Categories Endpoint', () => {
            const response = http.get(`${API_BASE_URL}/api/categories`, authHeaders);
            checkResponse(response, 200, 'Get categories');
        });
        
        sleep(sleepWithJitter(0.5));
        
        // Products endpoint
        group('Products Endpoint', () => {
            const response = http.get(`${API_BASE_URL}/api/products`, authHeaders);
            checkResponse(response, 200, 'Get products');
        });
    });
    
    sleep(sleepWithJitter(2));
}

export function teardown(data) {
    console.log('✅ Smoke test completed');
}