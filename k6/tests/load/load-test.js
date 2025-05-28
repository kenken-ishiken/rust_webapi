import http from 'k6/http';
import { check, group, sleep } from 'k6';
import exec from 'k6/execution';
import { API_BASE_URL, LOAD_TEST_STAGES, DEFAULT_THRESHOLDS } from '../../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../../utils/auth.js';
import { checkResponse, createTestItem, createTestProduct, createTestCategory, createTestUser, sleepWithJitter, parseResponse } from '../../utils/helpers.js';

// Load test - normal expected load
export let options = {
    stages: LOAD_TEST_STAGES,
    thresholds: {
        ...DEFAULT_THRESHOLDS,
        'http_req_duration{scenario:read}': ['p(95)<300', 'p(99)<500'],
        'http_req_duration{scenario:write}': ['p(95)<500', 'p(99)<1000'],
        'http_req_failed': ['rate<0.01'], // 1% error rate threshold
        'http_reqs': ['rate>200'],        // At least 200 requests per second
    },
};

export function setup() {
    const token = getAccessToken('testuser', 'testpass123');
    
    if (!token) {
        throw new Error('Failed to authenticate - load test cannot proceed');
    }
    
    console.log('✅ Load test setup completed');
    return { token };
}

export default function (data) {
    const token = data.token;
    const authHeaders = getAuthHeaders(token);
    
    // Simulate different user behaviors
    const scenario = Math.random();
    
    if (scenario < 0.6) {
        // 60% - Read-heavy users (browsing)
        readHeavyScenario(token);
    } else if (scenario < 0.85) {
        // 25% - Mixed read/write users
        mixedScenario(token);
    } else {
        // 15% - Write-heavy users (admins/content creators)
        writeHeavyScenario(token);
    }
    
    sleep(sleepWithJitter(1));
}

function readHeavyScenario(token) {
    group('Read Heavy Scenario', () => {
        // Browse categories
        const categoriesResponse = http.get(
            `${API_BASE_URL}/api/categories`,
            getAuthHeadersWithTags(token, { scenario: 'read', operation: 'browse_categories' })
        );
        checkResponse(categoriesResponse, 200, 'Browse categories');
        
        sleep(sleepWithJitter(0.5));
        
        // Search products
        const searchTerms = ['phone', 'laptop', 'camera', 'headphones', 'tablet'];
        const searchTerm = searchTerms[Math.floor(Math.random() * searchTerms.length)];
        
        const productsResponse = http.get(
            `${API_BASE_URL}/api/products?q=${searchTerm}&limit=20`,
            getAuthHeadersWithTags(token, { scenario: 'read', operation: 'search_products' })
        );
        checkResponse(productsResponse, 200, 'Search products');
        
        const products = parseResponse(productsResponse);
        
        // View individual products
        if (products && products.products && products.products.length > 0) {
            const randomProduct = products.products[Math.floor(Math.random() * products.products.length)];
            
            sleep(sleepWithJitter(0.3));
            
            const productDetailResponse = http.get(
                `${API_BASE_URL}/api/products/${randomProduct.id}`,
                getAuthHeadersWithTags(token, { scenario: 'read', operation: 'view_product' })
            );
            checkResponse(productDetailResponse, 200, 'View product detail');
        }
        
        sleep(sleepWithJitter(0.5));
        
        // Check items
        const itemsResponse = http.get(
            `${API_BASE_URL}/api/items`,
            getAuthHeadersWithTags(token, { scenario: 'read', operation: 'list_items' })
        );
        checkResponse(itemsResponse, 200, 'List items');
    });
}

function mixedScenario(token) {
    group('Mixed Read/Write Scenario', () => {
        // Create a category
        const category = createTestCategory();
        const categoryResponse = http.post(
            `${API_BASE_URL}/api/categories`,
            JSON.stringify(category),
            getAuthHeadersWithTags(token, { scenario: 'write', operation: 'create_category' })
        );
        
        let categoryId;
        if (categoryResponse.status === 201) {
            const created = parseResponse(categoryResponse);
            categoryId = created.id;
        }
        
        sleep(sleepWithJitter(0.5));
        
        // Create a product
        const product = createTestProduct();
        if (categoryId) {
            product.category_id = categoryId;
        }
        
        const productResponse = http.post(
            `${API_BASE_URL}/api/products`,
            JSON.stringify(product),
            getAuthHeadersWithTags(token, { scenario: 'write', operation: 'create_product' })
        );
        
        let productId;
        if (productResponse.status === 201) {
            const created = parseResponse(productResponse);
            productId = created.id;
        }
        
        sleep(sleepWithJitter(0.5));
        
        // Read operations
        http.get(
            `${API_BASE_URL}/api/products?category_id=${categoryId}`,
            getAuthHeadersWithTags(token, { scenario: 'read', operation: 'products_by_category' })
        );
        
        sleep(sleepWithJitter(0.3));
        
        // Update product
        if (productId) {
            const updateData = {
                price: Math.random() * 1000,
                stock_quantity: Math.floor(Math.random() * 100)
            };
            
            http.patch(
                `${API_BASE_URL}/api/products/${productId}`,
                JSON.stringify(updateData),
                getAuthHeadersWithTags(token, { scenario: 'write', operation: 'update_product' })
            );
        }
        
        sleep(sleepWithJitter(0.5));
        
        // Cleanup
        if (productId) {
            http.del(`${API_BASE_URL}/api/products/${productId}`, null, getAuthHeaders(token));
        }
        if (categoryId) {
            http.del(`${API_BASE_URL}/api/categories/${categoryId}`, null, getAuthHeaders(token));
        }
    });
}

function writeHeavyScenario(token) {
    group('Write Heavy Scenario', () => {
        // Create multiple items
        for (let i = 0; i < 3; i++) {
            const item = createTestItem();
            const itemResponse = http.post(
                `${API_BASE_URL}/api/items`,
                JSON.stringify(item),
                getAuthHeadersWithTags(token, { scenario: 'write', operation: 'create_item' })
            );
            
            if (itemResponse.status === 201) {
                const created = parseResponse(itemResponse);
                
                // Update the item
                sleep(sleepWithJitter(0.2));
                
                const updateData = {
                    name: `Updated ${item.name}`,
                    description: `Updated at ${new Date().toISOString()}`
                };
                
                http.put(
                    `${API_BASE_URL}/api/items/${created.id}`,
                    JSON.stringify(updateData),
                    getAuthHeadersWithTags(token, { scenario: 'write', operation: 'update_item' })
                );
                
                // Delete the item
                sleep(sleepWithJitter(0.2));
                
                http.del(
                    `${API_BASE_URL}/api/items/${created.id}`,
                    null,
                    getAuthHeadersWithTags(token, { scenario: 'write', operation: 'delete_item' })
                );
            }
            
            sleep(sleepWithJitter(0.3));
        }
        
        // Batch operations
        const productIds = [];
        for (let i = 0; i < 5; i++) {
            const product = createTestProduct();
            const response = http.post(
                `${API_BASE_URL}/api/products`,
                JSON.stringify(product),
                getAuthHeaders(token)
            );
            
            if (response.status === 201) {
                const created = parseResponse(response);
                productIds.push(created.id);
            }
        }
        
        if (productIds.length > 0) {
            // Batch update
            const updates = productIds.map(id => ({
                id: id,
                updates: {
                    price: Math.random() * 1000,
                    is_active: true
                }
            }));
            
            http.put(
                `${API_BASE_URL}/api/products/batch`,
                JSON.stringify({ updates }),
                getAuthHeadersWithTags(token, { scenario: 'write', operation: 'batch_update' })
            );
            
            // Cleanup
            productIds.forEach(id => {
                http.del(`${API_BASE_URL}/api/products/${id}`, null, getAuthHeaders(token));
            });
        }
    });
}

export function teardown(data) {
    console.log('✅ Load test completed');
    console.log(`Total VUs: ${exec.instance.vusActive}`);
}