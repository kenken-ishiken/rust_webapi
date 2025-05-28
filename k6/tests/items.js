import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { API_BASE_URL } from '../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../utils/auth.js';
import { checkResponse, createTestItem, sleepWithJitter, parseResponse } from '../utils/helpers.js';

// Test configuration
export let options = {
    thresholds: {
        'http_req_duration{endpoint:items}': ['p(95)<500', 'p(99)<1000'],
        'http_req_failed{endpoint:items}': ['rate<0.05'],
        'group_duration{group:CRUD Operations}': ['p(95)<2000'],
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
    
    // Test all item endpoints
    group('Item API Tests', () => {
        let createdItemId;
        
        group('CRUD Operations', () => {
            // GET all items
            group('Get All Items', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/items`,
                    getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'get_all' })
                );
                
                checkResponse(response, 200, 'Get all items');
                
                const items = parseResponse(response);
                if (items && Array.isArray(items)) {
                    console.log(`Found ${items.length} items`);
                }
            });
            
            sleep(sleepWithJitter(1));
            
            // CREATE new item
            group('Create Item', () => {
                const newItem = createTestItem();
                
                const response = http.post(
                    `${API_BASE_URL}/api/items`,
                    JSON.stringify(newItem),
                    getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'create' })
                );
                
                checkResponse(response, 201, 'Create item');
                
                const created = parseResponse(response);
                if (created && created.id) {
                    createdItemId = created.id;
                    console.log(`Created item with ID: ${createdItemId}`);
                }
            });
            
            sleep(sleepWithJitter(0.5));
            
            // GET single item
            if (createdItemId) {
                group('Get Single Item', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/items/${createdItemId}`,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'get_one' })
                    );
                    
                    checkResponse(response, 200, 'Get single item');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // UPDATE item
                group('Update Item', () => {
                    const updateData = {
                        name: `Updated Item ${Date.now()}`,
                        description: 'This item has been updated'
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/items/${createdItemId}`,
                        JSON.stringify(updateData),
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'update' })
                    );
                    
                    checkResponse(response, 200, 'Update item');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // DELETE item
                group('Delete Item', () => {
                    const response = http.del(
                        `${API_BASE_URL}/api/items/${createdItemId}`,
                        null,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'delete' })
                    );
                    
                    checkResponse(response, 200, 'Delete item');
                });
            }
        });
        
        // Test deletion features
        group('Deletion Features', () => {
            let testItemId;
            
            // Create item for deletion tests
            const newItem = createTestItem();
            const createResponse = http.post(
                `${API_BASE_URL}/api/items`,
                JSON.stringify(newItem),
                authHeaders
            );
            
            if (createResponse.status === 201) {
                const created = parseResponse(createResponse);
                testItemId = created.id;
                
                // Validate deletion
                group('Validate Deletion', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/products/${testItemId}/deletion-check`,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'validate_deletion' })
                    );
                    
                    checkResponse(response, 200, 'Validate deletion');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Logical delete
                group('Logical Delete', () => {
                    const response = http.del(
                        `${API_BASE_URL}/api/products/${testItemId}`,
                        null,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'logical_delete' })
                    );
                    
                    checkResponse(response, 200, 'Logical delete');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Get deleted items
                group('Get Deleted Items', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/products/deleted`,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'get_deleted' })
                    );
                    
                    checkResponse(response, 200, 'Get deleted items');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Restore item
                group('Restore Item', () => {
                    const response = http.post(
                        `${API_BASE_URL}/api/products/${testItemId}/restore`,
                        null,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'restore' })
                    );
                    
                    checkResponse(response, 200, 'Restore item');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Physical delete
                group('Physical Delete', () => {
                    const response = http.del(
                        `${API_BASE_URL}/api/products/${testItemId}/permanent`,
                        null,
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'physical_delete' })
                    );
                    
                    checkResponse(response, 200, 'Physical delete');
                });
            }
        });
        
        // Test batch operations
        group('Batch Operations', () => {
            // Create multiple items for batch testing
            const itemIds = [];
            for (let i = 0; i < 3; i++) {
                const item = createTestItem();
                const response = http.post(
                    `${API_BASE_URL}/api/items`,
                    JSON.stringify(item),
                    authHeaders
                );
                
                if (response.status === 201) {
                    const created = parseResponse(response);
                    itemIds.push(created.id);
                }
                sleep(0.1);
            }
            
            if (itemIds.length > 0) {
                // Batch delete
                group('Batch Delete', () => {
                    const batchRequest = {
                        ids: itemIds,
                        is_physical: false
                    };
                    
                    const response = http.del(
                        `${API_BASE_URL}/api/products/batch`,
                        JSON.stringify(batchRequest),
                        getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'batch_delete' })
                    );
                    
                    checkResponse(response, 200, 'Batch delete');
                });
            }
        });
        
        // Get deletion logs
        group('Deletion Logs', () => {
            const response = http.get(
                `${API_BASE_URL}/api/deletion-logs`,
                getAuthHeadersWithTags(token, { endpoint: 'items', operation: 'get_logs' })
            );
            
            checkResponse(response, 200, 'Get deletion logs');
        });
    });
    
    sleep(sleepWithJitter(2));
}