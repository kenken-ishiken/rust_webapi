import http from 'k6/http';
import { check, group, sleep } from 'k6';
import exec from 'k6/execution';
import { API_BASE_URL, STRESS_TEST_STAGES } from '../../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../../utils/auth.js';
import { checkResponse, createTestProduct, createTestCategory, sleepWithJitter, parseResponse, generateRandomNumber } from '../../utils/helpers.js';

// Stress test - beyond normal load to find breaking point
export let options = {
    stages: STRESS_TEST_STAGES,
    thresholds: {
        'http_req_duration': ['p(95)<2000', 'p(99)<5000'], // Relaxed for stress test
        'http_req_failed': ['rate<0.1'],                    // Allow up to 10% failure
        'http_reqs': ['rate>100'],                          // Still expect decent throughput
    },
};

export function setup() {
    const token = getAccessToken('testuser', 'testpass123');
    
    if (!token) {
        throw new Error('Failed to authenticate - stress test cannot proceed');
    }
    
    // Pre-create some data for stress testing
    const authHeaders = getAuthHeaders(token);
    const testData = {
        categoryIds: [],
        productIds: []
    };
    
    // Create test categories
    for (let i = 0; i < 10; i++) {
        const category = createTestCategory();
        const response = http.post(`${API_BASE_URL}/api/categories`, JSON.stringify(category), authHeaders);
        if (response.status === 201) {
            const created = parseResponse(response);
            testData.categoryIds.push(created.id);
        }
    }
    
    // Create test products
    for (let i = 0; i < 50; i++) {
        const product = createTestProduct();
        if (testData.categoryIds.length > 0) {
            product.category_id = testData.categoryIds[Math.floor(Math.random() * testData.categoryIds.length)];
        }
        const response = http.post(`${API_BASE_URL}/api/products`, JSON.stringify(product), authHeaders);
        if (response.status === 201) {
            const created = parseResponse(response);
            testData.productIds.push(created.id);
        }
    }
    
    console.log(`✅ Stress test setup completed with ${testData.categoryIds.length} categories and ${testData.productIds.length} products`);
    return { token, testData };
}

export default function (data) {
    const token = data.token;
    const testData = data.testData;
    
    // Aggressive test scenarios
    const scenario = Math.random();
    
    if (scenario < 0.3) {
        // 30% - Heavy concurrent reads
        heavyReadScenario(token, testData);
    } else if (scenario < 0.6) {
        // 30% - Heavy writes
        heavyWriteScenario(token, testData);
    } else if (scenario < 0.8) {
        // 20% - Complex queries
        complexQueryScenario(token, testData);
    } else {
        // 20% - Rapid fire mixed operations
        rapidFireScenario(token, testData);
    }
    
    // Minimal sleep to maintain pressure
    sleep(Math.random() * 0.5);
}

function heavyReadScenario(token, testData) {
    group('Heavy Read Scenario', () => {
        // Concurrent product reads
        const batch = http.batch([
            ['GET', `${API_BASE_URL}/api/products?limit=100`, null, getAuthHeadersWithTags(token, { scenario: 'heavy_read', operation: 'list_all' })],
            ['GET', `${API_BASE_URL}/api/categories/tree`, null, getAuthHeadersWithTags(token, { scenario: 'heavy_read', operation: 'category_tree' })],
            ['GET', `${API_BASE_URL}/api/products/reports/low-stock?threshold=50`, null, getAuthHeadersWithTags(token, { scenario: 'heavy_read', operation: 'low_stock' })],
            ['GET', `${API_BASE_URL}/api/products/reports/out-of-stock`, null, getAuthHeadersWithTags(token, { scenario: 'heavy_read', operation: 'out_of_stock' })],
        ]);
        
        // Random product detail reads
        if (testData.productIds.length > 0) {
            for (let i = 0; i < 5; i++) {
                const productId = testData.productIds[Math.floor(Math.random() * testData.productIds.length)];
                http.get(
                    `${API_BASE_URL}/api/products/${productId}`,
                    getAuthHeadersWithTags(token, { scenario: 'heavy_read', operation: 'product_detail' })
                );
            }
        }
    });
}

function heavyWriteScenario(token, testData) {
    group('Heavy Write Scenario', () => {
        // Rapid product creation
        const products = [];
        for (let i = 0; i < 10; i++) {
            const product = createTestProduct();
            if (testData.categoryIds.length > 0) {
                product.category_id = testData.categoryIds[Math.floor(Math.random() * testData.categoryIds.length)];
            }
            
            const response = http.post(
                `${API_BASE_URL}/api/products`,
                JSON.stringify(product),
                getAuthHeadersWithTags(token, { scenario: 'heavy_write', operation: 'create_product' })
            );
            
            if (response.status === 201) {
                const created = parseResponse(response);
                products.push(created.id);
            }
        }
        
        // Rapid updates
        products.forEach(productId => {
            const updateData = {
                price: generateRandomNumber(100, 10000),
                stock_quantity: generateRandomNumber(0, 1000)
            };
            
            http.patch(
                `${API_BASE_URL}/api/products/${productId}`,
                JSON.stringify(updateData),
                getAuthHeadersWithTags(token, { scenario: 'heavy_write', operation: 'update_product' })
            );
        });
        
        // Batch operations
        if (products.length > 5) {
            const batchRequest = {
                ids: products.slice(0, 5),
                is_physical: false
            };
            
            http.del(
                `${API_BASE_URL}/api/products/batch`,
                JSON.stringify(batchRequest),
                getAuthHeadersWithTags(token, { scenario: 'heavy_write', operation: 'batch_delete' })
            );
        }
        
        // Cleanup remaining
        products.slice(5).forEach(id => {
            http.del(`${API_BASE_URL}/api/products/${id}`, null, getAuthHeaders(token));
        });
    });
}

function complexQueryScenario(token, testData) {
    group('Complex Query Scenario', () => {
        // Complex search queries
        const complexQueries = [
            `q=test&min_price=100&max_price=1000&category_id=${testData.categoryIds[0]}&in_stock=true&limit=50`,
            `q=product&is_active=true&sort_by=price&order=desc&limit=100&offset=50`,
            `category_id=${testData.categoryIds[0]}&category_id=${testData.categoryIds[1]}&min_price=50`,
        ];
        
        complexQueries.forEach(query => {
            http.get(
                `${API_BASE_URL}/api/products?${query}`,
                getAuthHeadersWithTags(token, { scenario: 'complex_query', operation: 'complex_search' })
            );
        });
        
        // Deep category traversal
        if (testData.categoryIds.length > 0) {
            const categoryId = testData.categoryIds[0];
            
            // Get full path
            http.get(
                `${API_BASE_URL}/api/categories/${categoryId}/path`,
                getAuthHeadersWithTags(token, { scenario: 'complex_query', operation: 'category_path' })
            );
            
            // Get all children
            http.get(
                `${API_BASE_URL}/api/categories/${categoryId}/children?include_inactive=true`,
                getAuthHeadersWithTags(token, { scenario: 'complex_query', operation: 'category_children' })
            );
        }
        
        // Product history queries
        if (testData.productIds.length > 0) {
            testData.productIds.slice(0, 3).forEach(productId => {
                http.get(
                    `${API_BASE_URL}/api/products/${productId}/history?limit=100`,
                    getAuthHeadersWithTags(token, { scenario: 'complex_query', operation: 'product_history' })
                );
            });
        }
    });
}

function rapidFireScenario(token, testData) {
    group('Rapid Fire Scenario', () => {
        // No delays between operations
        for (let i = 0; i < 20; i++) {
            const operation = Math.random();
            
            if (operation < 0.4 && testData.productIds.length > 0) {
                // Read
                const productId = testData.productIds[Math.floor(Math.random() * testData.productIds.length)];
                http.get(`${API_BASE_URL}/api/products/${productId}`, getAuthHeaders(token));
            } else if (operation < 0.7) {
                // Write
                const product = createTestProduct();
                const response = http.post(`${API_BASE_URL}/api/products`, JSON.stringify(product), getAuthHeaders(token));
                if (response.status === 201) {
                    const created = parseResponse(response);
                    // Immediate delete
                    http.del(`${API_BASE_URL}/api/products/${created.id}`, null, getAuthHeaders(token));
                }
            } else {
                // Update
                if (testData.productIds.length > 0) {
                    const productId = testData.productIds[Math.floor(Math.random() * testData.productIds.length)];
                    const updateData = { price: generateRandomNumber(100, 1000) };
                    http.patch(`${API_BASE_URL}/api/products/${productId}`, JSON.stringify(updateData), getAuthHeaders(token));
                }
            }
        }
    });
}

export function teardown(data) {
    const token = data.token;
    const authHeaders = getAuthHeaders(token);
    
    // Cleanup test data
    console.log('🧹 Cleaning up stress test data...');
    
    // Delete all test products
    data.testData.productIds.forEach(id => {
        http.del(`${API_BASE_URL}/api/products/${id}`, null, authHeaders);
    });
    
    // Delete all test categories
    data.testData.categoryIds.forEach(id => {
        http.del(`${API_BASE_URL}/api/categories/${id}`, null, authHeaders);
    });
    
    console.log('✅ Stress test completed');
    console.log(`Peak VUs: ${exec.instance.vusActive}`);
}