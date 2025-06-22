// API endpoints configuration
export const API_BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080';
export const KEYCLOAK_URL = __ENV.KEYCLOAK_URL || 'http://localhost:8081';
export const KEYCLOAK_REALM = __ENV.KEYCLOAK_REALM || 'rust-webapi';
export const KEYCLOAK_CLIENT_ID = __ENV.KEYCLOAK_CLIENT_ID || 'api-client';
export const KEYCLOAK_CLIENT_SECRET = __ENV.KEYCLOAK_CLIENT_SECRET || '';

// Test users
export const TEST_USER = {
    username: __ENV.TEST_USERNAME || 'testuser',
    password: __ENV.TEST_PASSWORD || 'testpass123'
};

// Test thresholds
export const DEFAULT_THRESHOLDS = {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    http_req_failed: ['rate<0.1'],
    http_reqs: ['rate>100']
};

// Test stages for different scenarios
export const SMOKE_TEST_STAGES = [
    { duration: '30s', target: 5 },   // Ramp up to 5 users
    { duration: '1m', target: 5 },    // Stay at 5 users
    { duration: '30s', target: 0 },   // Ramp down to 0 users
];

export const LOAD_TEST_STAGES = [
    { duration: '2m', target: 50 },   // Ramp up to 50 users
    { duration: '5m', target: 50 },   // Stay at 50 users
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 0 },    // Ramp down to 0 users
];

export const STRESS_TEST_STAGES = [
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 200 },  // Ramp up to 200 users
    { duration: '5m', target: 200 },  // Stay at 200 users
    { duration: '2m', target: 300 },  // Ramp up to 300 users
    { duration: '5m', target: 300 },  // Stay at 300 users
    { duration: '2m', target: 0 },    // Ramp down to 0 users
];

export const SPIKE_TEST_STAGES = [
    { duration: '1m', target: 50 },   // Ramp up to 50 users
    { duration: '30s', target: 500 }, // Spike to 500 users
    { duration: '30s', target: 50 },  // Drop back to 50 users
    { duration: '2m', target: 50 },   // Stay at 50 users
    { duration: '1m', target: 0 },    // Ramp down to 0 users
];

// Response time requirements
export const SLA = {
    P95_RESPONSE_TIME: 500,  // 95th percentile should be under 500ms
    P99_RESPONSE_TIME: 1000, // 99th percentile should be under 1000ms
    ERROR_RATE: 0.01,        // Error rate should be under 1%
};

// SLA validation requirements from REMAINING_IMPROVEMENTS.md
export const STRICT_SLA = {
    P95_RESPONSE_TIME: 250,    // 95th percentile should be under 250ms
    P99_RESPONSE_TIME: 400,    // 99th percentile should be under 400ms
    ERROR_RATE: 0.001,         // Error rate should be under 0.1%
    MIN_THROUGHPUT: 500,       // Minimum 500 requests per second
    CONCURRENT_USERS: 1000,    // Support 1000 concurrent users
};

// SLA test stages for ramping up to 1000 users
export const SLA_TEST_STAGES = [
    { duration: '1m', target: 100 },   // Warm up
    { duration: '2m', target: 500 },   // Ramp to 500 users
    { duration: '2m', target: 1000 },  // Ramp to 1000 users
    { duration: '5m', target: 1000 },  // Maintain 1000 users
    { duration: '1m', target: 0 },     // Ramp down
];