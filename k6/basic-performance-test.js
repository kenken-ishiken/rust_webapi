import http from 'k6/http';
import { check } from 'k6';

export let options = {
    vus: 100,
    duration: '30s',
    thresholds: {
        http_req_duration: ['p(95)<250'],
        http_req_failed: ['rate<0.001'],
    },
};

export default function () {
    // Test health endpoint (no auth required)
    const response = http.get('http://localhost:8080/api/health');
    
    check(response, {
        'status is 200': (r) => r.status === 200,
        'response body is OK': (r) => r.body === 'OK',
    });
} 