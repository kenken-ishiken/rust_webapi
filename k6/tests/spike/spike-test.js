import http from 'k6/http';
import { check, group, sleep } from 'k6';
import exec from 'k6/execution';
import { API_BASE_URL, SPIKE_TEST_STAGES } from '../../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../../utils/auth.js';
import { checkResponse, createTestProduct, sleepWithJitter, parseResponse } from '../../utils/helpers.js';

// Spike test - sudden increase/decrease in traffic
export let options = {
    stages: SPIKE_TEST_STAGES,
    thresholds: {
        'http_req_duration': ['p(95)<3000', 'p(99)<10000'], // Very relaxed for spike
        'http_req_failed': ['rate<0.2'],                     // Allow up to 20% failure during spike
        'http_reqs': ['rate>50'],                            // Lower expectation during spike
    },
};

export function setup() {
    const token = getAccessToken('testuser', 'testpass123');
    
    if (!token) {
        throw new Error('Failed to authenticate - spike test cannot proceed');
    }
    
    console.log('✅ Spike test setup completed');
    console.log('⚡ Test will spike from 50 to 500 users suddenly');
    return { token };
}

export default function (data) {
    const token = data.token;
    const currentStage = exec.scenario.iterationInTest;
    const vus = exec.instance.vusActive;
    
    // Different behavior based on current load
    if (vus > 300) {
        // During spike - simple operations only
        spikeScenario(token);
    } else if (vus > 100) {
        // Ramping up/down - mixed operations
        transitionScenario(token);
    } else {
        // Normal load - regular operations
        normalScenario(token);
    }
    
    // Adjust sleep based on load
    const sleepTime = vus > 300 ? 0.1 : sleepWithJitter(0.5);
    sleep(sleepTime);
}

function spikeScenario(token) {
    group('Spike Load Operations', () => {
        // During spike, focus on simple, fast operations
        const operations = [
            () => {
                // Simple GET
                http.get(
                    `${API_BASE_URL}/api/health`,
                    getAuthHeadersWithTags(token, { scenario: 'spike', operation: 'health_check' })
                );
            },
            () => {
                // List products (limited)
                http.get(
                    `${API_BASE_URL}/api/products?limit=10`,
                    getAuthHeadersWithTags(token, { scenario: 'spike', operation: 'list_products_limited' })
                );
            },
            () => {
                // List categories
                http.get(
                    `${API_BASE_URL}/api/categories`,
                    getAuthHeadersWithTags(token, { scenario: 'spike', operation: 'list_categories' })
                );
            },
            () => {
                // Simple search
                http.get(
                    `${API_BASE_URL}/api/products?q=test&limit=5`,
                    getAuthHeadersWithTags(token, { scenario: 'spike', operation: 'simple_search' })
                );
            }
        ];
        
        // Execute random operation
        const operation = operations[Math.floor(Math.random() * operations.length)];
        operation();
    });
}

function transitionScenario(token) {
    group('Transition Operations', () => {
        // Mixed read/write during ramp up/down
        const scenario = Math.random();
        
        if (scenario < 0.7) {
            // 70% reads
            const response = http.get(
                `${API_BASE_URL}/api/products?limit=20`,
                getAuthHeadersWithTags(token, { scenario: 'transition', operation: 'read_products' })
            );
            
            if (response.status === 200) {
                const products = parseResponse(response);
                if (products && products.products && products.products.length > 0) {
                    const product = products.products[0];
                    http.get(
                        `${API_BASE_URL}/api/products/${product.id}`,
                        getAuthHeadersWithTags(token, { scenario: 'transition', operation: 'read_product_detail' })
                    );
                }
            }
        } else {
            // 30% writes
            const product = createTestProduct();
            const response = http.post(
                `${API_BASE_URL}/api/products`,
                JSON.stringify(product),
                getAuthHeadersWithTags(token, { scenario: 'transition', operation: 'create_product' })
            );
            
            if (response.status === 201) {
                const created = parseResponse(response);
                // Quick cleanup
                setTimeout(() => {
                    http.del(`${API_BASE_URL}/api/products/${created.id}`, null, getAuthHeaders(token));
                }, 5000);
            }
        }
    });
}

function normalScenario(token) {
    group('Normal Load Operations', () => {
        // Full range of operations during normal load
        
        // Category operations
        const categoriesResponse = http.get(
            `${API_BASE_URL}/api/categories/tree`,
            getAuthHeadersWithTags(token, { scenario: 'normal', operation: 'category_tree' })
        );
        
        sleep(sleepWithJitter(0.3));
        
        // Product operations
        const product = createTestProduct();
        const createResponse = http.post(
            `${API_BASE_URL}/api/products`,
            JSON.stringify(product),
            getAuthHeadersWithTags(token, { scenario: 'normal', operation: 'create_product' })
        );
        
        if (createResponse.status === 201) {
            const created = parseResponse(createResponse);
            
            // Update price
            http.put(
                `${API_BASE_URL}/api/products/${created.id}/price`,
                JSON.stringify({ price: 99.99 }),
                getAuthHeadersWithTags(token, { scenario: 'normal', operation: 'update_price' })
            );
            
            sleep(sleepWithJitter(0.2));
            
            // Update inventory
            http.put(
                `${API_BASE_URL}/api/products/${created.id}/inventory`,
                JSON.stringify({ quantity: 100 }),
                getAuthHeadersWithTags(token, { scenario: 'normal', operation: 'update_inventory' })
            );
            
            // Cleanup
            http.del(`${API_BASE_URL}/api/products/${created.id}`, null, getAuthHeaders(token));
        }
        
        // Reports
        http.get(
            `${API_BASE_URL}/api/products/reports/low-stock`,
            getAuthHeadersWithTags(token, { scenario: 'normal', operation: 'low_stock_report' })
        );
    });
}

export function handleSummary(data) {
    // Custom summary to highlight spike impact
    const maxVus = data.metrics.vus_max ? data.metrics.vus_max.values.max : 0;
    const avgDuration = data.metrics.http_req_duration ? data.metrics.http_req_duration.values.avg : 0;
    const errorRate = data.metrics.http_req_failed ? data.metrics.http_req_failed.values.rate : 0;
    
    console.log('\n=== Spike Test Summary ===');
    console.log(`Maximum concurrent users: ${maxVus}`);
    console.log(`Average response time: ${avgDuration.toFixed(2)}ms`);
    console.log(`Error rate: ${(errorRate * 100).toFixed(2)}%`);
    console.log(`Total requests: ${data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0}`);
    
    // Check if system recovered after spike
    const p95Duration = data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0;
    if (p95Duration < 1000) {
        console.log('✅ System recovered well after spike');
    } else {
        console.log('⚠️  System showing signs of stress after spike');
    }
    
    return {
        'summary.json': JSON.stringify(data, null, 2),
    };
}

export function teardown(data) {
    console.log('✅ Spike test completed');
}