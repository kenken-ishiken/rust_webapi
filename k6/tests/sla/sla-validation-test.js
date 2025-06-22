import http from 'k6/http';
import { check, group, sleep } from 'k6';
import exec from 'k6/execution';
import { API_BASE_URL } from '../../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../../utils/auth.js';
import { checkResponse, createTestItem, createTestProduct, createTestCategory, sleepWithJitter, parseResponse } from '../../utils/helpers.js';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics for SLA tracking
const slaViolations = new Counter('sla_violations');
const responseTime95 = new Trend('response_time_p95');
const throughputRate = new Rate('throughput_achieved');

// SLA Validation Test - meets the criteria from REMAINING_IMPROVEMENTS.md
export let options = {
    scenarios: {
        // Main scenario: maintain 1000 concurrent users
        constant_load: {
            executor: 'constant-vus',
            vus: 1000,
            duration: '5m',
            gracefulStop: '30s',
        },
        // Ramp up scenario to reach 1000 users
        ramp_up: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '2m', target: 1000 }, // Ramp up to 1000 users
            ],
            gracefulRampDown: '30s',
            startTime: '0s',
        }
    },
    thresholds: {
        // SLA Requirements from REMAINING_IMPROVEMENTS.md
        'http_req_duration': [
            'p(95)<250',  // 95 percentile < 250ms
            'p(99)<400',  // Additional constraint for stability
        ],
        'http_req_failed': ['rate<0.001'],  // Error rate < 0.1%
        'http_reqs': ['rate>500'],          // Throughput > 500 req/s with 1000 users
        
        // Custom SLA metrics
        'sla_violations': ['count<100'],    // Less than 100 SLA violations allowed
        'response_time_p95': ['p(95)<250'], // Track 95th percentile separately
        'throughput_achieved': ['rate>0.99'], // 99% of requests should complete
        
        // Endpoint-specific thresholds
        'http_req_duration{endpoint:products_list}': ['p(95)<200'],
        'http_req_duration{endpoint:product_detail}': ['p(95)<150'],
        'http_req_duration{endpoint:categories_list}': ['p(95)<200'],
        'http_req_duration{endpoint:items_list}': ['p(95)<200'],
        'http_req_duration{endpoint:create_item}': ['p(95)<300'],
        'http_req_duration{endpoint:update_product}': ['p(95)<300'],
    },
};

export function setup() {
    const token = getAccessToken('testuser', 'testpass123');
    
    if (!token) {
        throw new Error('Failed to authenticate - SLA validation test cannot proceed');
    }
    
    console.log('✅ SLA validation test setup completed');
    console.log('📊 Testing against SLA criteria:');
    console.log('   - 95th percentile response time < 250ms');
    console.log('   - Error rate < 0.1%');
    console.log('   - Throughput > 500 req/s with 1000 concurrent users');
    
    return { token, startTime: new Date() };
}

export default function (data) {
    const token = data.token;
    
    // Weighted scenario distribution for realistic load
    const scenario = Math.random();
    
    if (scenario < 0.4) {
        // 40% - Product browsing (most common)
        productBrowsingScenario(token);
    } else if (scenario < 0.65) {
        // 25% - Category navigation
        categoryNavigationScenario(token);
    } else if (scenario < 0.85) {
        // 20% - Item operations
        itemOperationsScenario(token);
    } else if (scenario < 0.95) {
        // 10% - Mixed read operations
        mixedReadScenario(token);
    } else {
        // 5% - Write operations
        writeOperationsScenario(token);
    }
    
    // Minimal sleep to maximize throughput
    sleep(sleepWithJitter(0.1));
}

function productBrowsingScenario(token) {
    group('Product Browsing', () => {
        // List products with pagination
        const listStart = Date.now();
        const productsResponse = http.get(
            `${API_BASE_URL}/api/products?limit=20&offset=0`,
            {
                headers: getAuthHeaders(token).headers,
                tags: { endpoint: 'products_list' }
            }
        );
        const listDuration = Date.now() - listStart;
        
        // Track SLA metrics
        responseTime95.add(listDuration);
        if (listDuration > 250) {
            slaViolations.add(1);
        }
        throughputRate.add(productsResponse.status === 200);
        
        checkResponse(productsResponse, 200, 'List products');
        
        const products = parseResponse(productsResponse);
        
        // View product details for 2-3 products
        if (products && products.products && products.products.length > 0) {
            const viewCount = Math.min(Math.floor(Math.random() * 2) + 2, products.products.length);
            
            for (let i = 0; i < viewCount; i++) {
                sleep(sleepWithJitter(0.05)); // Very short delay between requests
                
                const product = products.products[i];
                const detailStart = Date.now();
                const detailResponse = http.get(
                    `${API_BASE_URL}/api/products/${product.id}`,
                    {
                        headers: getAuthHeaders(token).headers,
                        tags: { endpoint: 'product_detail' }
                    }
                );
                const detailDuration = Date.now() - detailStart;
                
                responseTime95.add(detailDuration);
                if (detailDuration > 250) {
                    slaViolations.add(1);
                }
                throughputRate.add(detailResponse.status === 200);
                
                checkResponse(detailResponse, 200, 'View product detail');
            }
        }
    });
}

function categoryNavigationScenario(token) {
    group('Category Navigation', () => {
        // Get category tree
        const treeStart = Date.now();
        const categoriesResponse = http.get(
            `${API_BASE_URL}/api/categories/tree`,
            {
                headers: getAuthHeaders(token).headers,
                tags: { endpoint: 'categories_list' }
            }
        );
        const treeDuration = Date.now() - treeStart;
        
        responseTime95.add(treeDuration);
        if (treeDuration > 250) {
            slaViolations.add(1);
        }
        throughputRate.add(categoriesResponse.status === 200);
        
        checkResponse(categoriesResponse, 200, 'Get category tree');
        
        // Browse products in a category
        const categories = parseResponse(categoriesResponse);
        if (categories && categories.length > 0) {
            const randomCategory = categories[Math.floor(Math.random() * categories.length)];
            
            sleep(sleepWithJitter(0.05));
            
            const productsStart = Date.now();
            const productsInCategoryResponse = http.get(
                `${API_BASE_URL}/api/products?category_id=${randomCategory.id}&limit=10`,
                {
                    headers: getAuthHeaders(token).headers,
                    tags: { endpoint: 'products_list' }
                }
            );
            const productsDuration = Date.now() - productsStart;
            
            responseTime95.add(productsDuration);
            if (productsDuration > 250) {
                slaViolations.add(1);
            }
            throughputRate.add(productsInCategoryResponse.status === 200);
            
            checkResponse(productsInCategoryResponse, 200, 'Products in category');
        }
    });
}

function itemOperationsScenario(token) {
    group('Item Operations', () => {
        // List items
        const listStart = Date.now();
        const itemsResponse = http.get(
            `${API_BASE_URL}/api/items?limit=20`,
            {
                headers: getAuthHeaders(token).headers,
                tags: { endpoint: 'items_list' }
            }
        );
        const listDuration = Date.now() - listStart;
        
        responseTime95.add(listDuration);
        if (listDuration > 250) {
            slaViolations.add(1);
        }
        throughputRate.add(itemsResponse.status === 200);
        
        checkResponse(itemsResponse, 200, 'List items');
        
        // View specific items
        const items = parseResponse(itemsResponse);
        if (items && items.length > 0) {
            const item = items[Math.floor(Math.random() * items.length)];
            
            sleep(sleepWithJitter(0.05));
            
            const detailStart = Date.now();
            const itemDetailResponse = http.get(
                `${API_BASE_URL}/api/items/${item.id}`,
                {
                    headers: getAuthHeaders(token).headers,
                    tags: { endpoint: 'item_detail' }
                }
            );
            const detailDuration = Date.now() - detailStart;
            
            responseTime95.add(detailDuration);
            if (detailDuration > 250) {
                slaViolations.add(1);
            }
            throughputRate.add(itemDetailResponse.status === 200);
            
            checkResponse(itemDetailResponse, 200, 'View item detail');
        }
    });
}

function mixedReadScenario(token) {
    group('Mixed Read Operations', () => {
        // Perform multiple read operations in sequence
        const operations = [
            { url: '/api/products?limit=10', tag: 'products_list' },
            { url: '/api/categories', tag: 'categories_list' },
            { url: '/api/items?limit=10', tag: 'items_list' },
            { url: '/api/users?limit=10', tag: 'users_list' },
        ];
        
        operations.forEach((op, index) => {
            if (index > 0) {
                sleep(sleepWithJitter(0.05));
            }
            
            const start = Date.now();
            const response = http.get(
                `${API_BASE_URL}${op.url}`,
                {
                    headers: getAuthHeaders(token).headers,
                    tags: { endpoint: op.tag }
                }
            );
            const duration = Date.now() - start;
            
            responseTime95.add(duration);
            if (duration > 250) {
                slaViolations.add(1);
            }
            throughputRate.add(response.status === 200);
            
            checkResponse(response, 200, `Mixed read: ${op.tag}`);
        });
    });
}

function writeOperationsScenario(token) {
    group('Write Operations', () => {
        // Create an item
        const item = createTestItem();
        const createStart = Date.now();
        const createResponse = http.post(
            `${API_BASE_URL}/api/items`,
            JSON.stringify(item),
            {
                headers: getAuthHeaders(token).headers,
                tags: { endpoint: 'create_item' }
            }
        );
        const createDuration = Date.now() - createStart;
        
        responseTime95.add(createDuration);
        if (createDuration > 250) {
            slaViolations.add(1);
        }
        throughputRate.add(createResponse.status === 201);
        
        if (createResponse.status === 201) {
            const created = parseResponse(createResponse);
            
            sleep(sleepWithJitter(0.05));
            
            // Update the item
            const updateData = {
                name: `Updated ${item.name}`,
                description: `Updated at ${new Date().toISOString()}`
            };
            
            const updateStart = Date.now();
            const updateResponse = http.put(
                `${API_BASE_URL}/api/items/${created.id}`,
                JSON.stringify(updateData),
                {
                    headers: getAuthHeaders(token).headers,
                    tags: { endpoint: 'update_item' }
                }
            );
            const updateDuration = Date.now() - updateStart;
            
            responseTime95.add(updateDuration);
            if (updateDuration > 250) {
                slaViolations.add(1);
            }
            throughputRate.add(updateResponse.status === 200);
            
            // Clean up
            sleep(sleepWithJitter(0.05));
            http.del(`${API_BASE_URL}/api/items/${created.id}`, null, getAuthHeaders(token));
        }
    });
}

export function teardown(data) {
    const duration = (new Date() - data.startTime) / 1000;
    
    console.log('✅ SLA validation test completed');
    console.log(`📊 Test duration: ${duration}s`);
    console.log(`👥 Peak VUs: ${exec.instance.vusMax}`);
    
    // Summary will be printed by k6 with threshold results
}