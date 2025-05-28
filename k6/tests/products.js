import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { API_BASE_URL } from '../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../utils/auth.js';
import { checkResponse, createTestProduct, sleepWithJitter, parseResponse, generateRandomNumber } from '../utils/helpers.js';

// Test configuration
export let options = {
    thresholds: {
        'http_req_duration{endpoint:products}': ['p(95)<500', 'p(99)<1000'],
        'http_req_failed{endpoint:products}': ['rate<0.05'],
        'group_duration{group:Product CRUD}': ['p(95)<2000'],
        'group_duration{group:Price and Inventory}': ['p(95)<1000'],
    },
};

// Test setup
export function setup() {
    const token = getAccessToken('testuser', 'testpass123');
    
    if (!token) {
        throw new Error('Failed to authenticate');
    }
    
    return { token };
}

export default function (data) {
    const token = data.token;
    const authHeaders = getAuthHeaders(token);
    
    group('Product API Tests', () => {
        let createdProductId;
        let createdProductSku;
        
        group('Product CRUD', () => {
            // Search products
            group('Search Products', () => {
                const searchQueries = [
                    { q: 'test' },
                    { q: 'product', limit: 10, offset: 0 },
                    { category_id: 'cat_123', is_active: true },
                    { min_price: 10, max_price: 100 },
                    { in_stock: true }
                ];
                
                const query = searchQueries[Math.floor(Math.random() * searchQueries.length)];
                const queryString = new URLSearchParams(query).toString();
                
                const response = http.get(
                    `${API_BASE_URL}/api/products?${queryString}`,
                    getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'search' })
                );
                
                checkResponse(response, 200, 'Search products');
                
                const result = parseResponse(response);
                if (result && result.products) {
                    console.log(`Found ${result.total} products`);
                }
            });
            
            sleep(sleepWithJitter(1));
            
            // Create product
            group('Create Product', () => {
                const newProduct = createTestProduct();
                
                const response = http.post(
                    `${API_BASE_URL}/api/products`,
                    JSON.stringify(newProduct),
                    getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'create' })
                );
                
                if (response.status === 201) {
                    checkResponse(response, 201, 'Create product');
                    const created = parseResponse(response);
                    if (created) {
                        createdProductId = created.id;
                        createdProductSku = created.sku;
                        console.log(`Created product: ${createdProductId} (SKU: ${createdProductSku})`);
                    }
                } else if (response.status === 409) {
                    console.log('Product SKU already exists, continuing with tests...');
                }
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Get product by ID
            if (createdProductId) {
                group('Get Product by ID', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/products/${createdProductId}`,
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'get_by_id' })
                    );
                    
                    checkResponse(response, 200, 'Get product by ID');
                });
                
                sleep(sleepWithJitter(0.5));
            }
            
            // Get product by SKU
            if (createdProductSku) {
                group('Get Product by SKU', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/products/sku/${createdProductSku}`,
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'get_by_sku' })
                    );
                    
                    checkResponse(response, 200, 'Get product by SKU');
                });
                
                sleep(sleepWithJitter(0.5));
            }
            
            // Update product
            if (createdProductId) {
                group('Update Product', () => {
                    const updateData = {
                        name: `Updated Product ${Date.now()}`,
                        description: 'This product has been updated',
                        price: generateRandomNumber(50, 500),
                        stock_quantity: generateRandomNumber(10, 100),
                        is_active: true
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/products/${createdProductId}`,
                        JSON.stringify(updateData),
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'update' })
                    );
                    
                    checkResponse(response, 200, 'Update product');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Patch product
                group('Patch Product', () => {
                    const patchData = {
                        description: 'Partially updated description',
                        is_active: true
                    };
                    
                    const response = http.patch(
                        `${API_BASE_URL}/api/products/${createdProductId}`,
                        JSON.stringify(patchData),
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'patch' })
                    );
                    
                    checkResponse(response, 200, 'Patch product');
                });
            }
        });
        
        // Price and Inventory operations
        if (createdProductId) {
            group('Price and Inventory', () => {
                // Update price
                group('Update Price', () => {
                    const priceData = {
                        price: generateRandomNumber(100, 1000),
                        compare_at_price: generateRandomNumber(1200, 1500),
                        cost: generateRandomNumber(50, 90)
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/products/${createdProductId}/price`,
                        JSON.stringify(priceData),
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'update_price' })
                    );
                    
                    checkResponse(response, 200, 'Update price');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Update inventory
                group('Update Inventory', () => {
                    const inventoryData = {
                        quantity: generateRandomNumber(50, 200),
                        reserved_quantity: generateRandomNumber(0, 20),
                        warehouse_location: `W-${generateRandomNumber(1, 10)}-${generateRandomNumber(1, 100)}`
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/products/${createdProductId}/inventory`,
                        JSON.stringify(inventoryData),
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'update_inventory' })
                    );
                    
                    checkResponse(response, 200, 'Update inventory');
                });
            });
        }
        
        // Image operations
        if (createdProductId) {
            group('Image Operations', () => {
                let imageId;
                
                // Add image
                group('Add Image', () => {
                    const imageData = {
                        url: `https://example.com/products/${createdProductId}/image-${Date.now()}.jpg`,
                        alt_text: 'Product image',
                        is_primary: true,
                        sort_order: 1
                    };
                    
                    const response = http.post(
                        `${API_BASE_URL}/api/products/${createdProductId}/images`,
                        JSON.stringify(imageData),
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'add_image' })
                    );
                    
                    if (response.status === 201) {
                        checkResponse(response, 201, 'Add image');
                        const image = parseResponse(response);
                        if (image) {
                            imageId = image.id;
                        }
                    }
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Update image
                if (imageId) {
                    group('Update Image', () => {
                        const updateImageData = {
                            alt_text: 'Updated product image',
                            sort_order: 2
                        };
                        
                        const response = http.put(
                            `${API_BASE_URL}/api/products/${createdProductId}/images/${imageId}`,
                            JSON.stringify(updateImageData),
                            getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'update_image' })
                        );
                        
                        checkResponse(response, 200, 'Update image');
                    });
                    
                    sleep(sleepWithJitter(0.5));
                    
                    // Set main image
                    group('Set Main Image', () => {
                        const response = http.put(
                            `${API_BASE_URL}/api/products/${createdProductId}/images/${imageId}/main`,
                            null,
                            getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'set_main_image' })
                        );
                        
                        checkResponse(response, 200, 'Set main image');
                    });
                    
                    sleep(sleepWithJitter(0.5));
                    
                    // Delete image
                    group('Delete Image', () => {
                        const response = http.del(
                            `${API_BASE_URL}/api/products/${createdProductId}/images/${imageId}`,
                            null,
                            getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'delete_image' })
                        );
                        
                        checkResponse(response, 204, 'Delete image');
                    });
                }
            });
        }
        
        // Batch operations
        group('Batch Operations', () => {
            // Create multiple products for batch testing
            const productIds = [];
            for (let i = 0; i < 3; i++) {
                const product = createTestProduct();
                const response = http.post(
                    `${API_BASE_URL}/api/products`,
                    JSON.stringify(product),
                    authHeaders
                );
                
                if (response.status === 201) {
                    const created = parseResponse(response);
                    productIds.push(created.id);
                }
                sleep(0.1);
            }
            
            if (productIds.length > 0) {
                // Batch update
                group('Batch Update', () => {
                    const updates = productIds.map(id => ({
                        id: id,
                        updates: {
                            price: generateRandomNumber(100, 500),
                            stock_quantity: generateRandomNumber(10, 100),
                            is_active: true
                        }
                    }));
                    
                    const batchRequest = { updates };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/products/batch`,
                        JSON.stringify(batchRequest),
                        getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'batch_update' })
                    );
                    
                    checkResponse(response, 200, 'Batch update');
                });
                
                // Clean up: delete created products
                productIds.forEach(id => {
                    http.del(`${API_BASE_URL}/api/products/${id}`, null, authHeaders);
                });
            }
        });
        
        // Reports
        group('Reports', () => {
            // Low stock report
            group('Low Stock Report', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/products/reports/low-stock?threshold=50`,
                    getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'low_stock_report' })
                );
                
                checkResponse(response, 200, 'Low stock report');
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Out of stock report
            group('Out of Stock Report', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/products/reports/out-of-stock`,
                    getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'out_of_stock_report' })
                );
                
                checkResponse(response, 200, 'Out of stock report');
            });
        });
        
        // History
        if (createdProductId) {
            group('Product History', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/products/${createdProductId}/history?limit=10`,
                    getAuthHeadersWithTags(token, { endpoint: 'products', operation: 'get_history' })
                );
                
                checkResponse(response, 200, 'Get product history');
            });
            
            // Clean up: delete the created product
            group('Cleanup', () => {
                const response = http.del(
                    `${API_BASE_URL}/api/products/${createdProductId}`,
                    null,
                    authHeaders
                );
                
                console.log(`Cleaned up product ${createdProductId}`);
            });
        }
    });
    
    sleep(sleepWithJitter(2));
}