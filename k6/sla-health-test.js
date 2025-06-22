import http from 'k6/http';
import { check } from 'k6';

export let options = {
    stages: [
        { duration: '30s', target: 100 },   // Ramp up to 100 users
        { duration: '30s', target: 500 },   // Ramp up to 500 users
        { duration: '1m', target: 1000 },   // Ramp up to 1000 users
        { duration: '3m', target: 1000 },   // Stay at 1000 users for 3 minutes
        { duration: '1m', target: 0 },      // Ramp down to 0 users
    ],
    thresholds: {
        http_req_duration: ['p(95)<250'],   // 95th percentile < 250ms
        http_req_failed: ['rate<0.001'],    // Error rate < 0.1%
        http_reqs: ['rate>500'],            // Throughput > 500 req/s
    },
};

export default function () {
    // Test health endpoint (no auth required)
    const response = http.get('http://localhost:8080/api/health');
    
    check(response, {
        'status is 200': (r) => r.status === 200,
        'response body is OK': (r) => r.body === 'OK',
        'response time < 250ms': (r) => r.timings.duration < 250,
    });
} 