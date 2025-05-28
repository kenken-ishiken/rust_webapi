import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { API_BASE_URL } from '../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../utils/auth.js';
import { checkResponse, createTestCategory, sleepWithJitter, parseResponse } from '../utils/helpers.js';

// Test configuration
export let options = {
    thresholds: {
        'http_req_duration{endpoint:categories}': ['p(95)<500', 'p(99)<1000'],
        'http_req_failed{endpoint:categories}': ['rate<0.05'],
        'group_duration{group:Category CRUD}': ['p(95)<2000'],
        'group_duration{group:Category Hierarchy}': ['p(95)<1500'],
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
    
    group('Category API Tests', () => {
        let rootCategoryId;
        let childCategoryId;
        
        group('Category CRUD', () => {
            // Get all categories
            group('Get All Categories', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/categories`,
                    getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_all' })
                );
                
                checkResponse(response, 200, 'Get all categories');
                
                const result = parseResponse(response);
                if (result && result.categories) {
                    console.log(`Found ${result.total} categories`);
                }
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Get category tree
            group('Get Category Tree', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/categories/tree`,
                    getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_tree' })
                );
                
                checkResponse(response, 200, 'Get category tree');
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Create root category
            group('Create Root Category', () => {
                const newCategory = createTestCategory();
                
                const response = http.post(
                    `${API_BASE_URL}/api/categories`,
                    JSON.stringify(newCategory),
                    getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'create' })
                );
                
                if (response.status === 201) {
                    checkResponse(response, 201, 'Create root category');
                    const created = parseResponse(response);
                    if (created && created.id) {
                        rootCategoryId = created.id;
                        console.log(`Created root category: ${rootCategoryId}`);
                    }
                } else if (response.status === 409) {
                    console.log('Category name already exists, continuing...');
                }
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Get single category
            if (rootCategoryId) {
                group('Get Single Category', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/categories/${rootCategoryId}`,
                        getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_one' })
                    );
                    
                    checkResponse(response, 200, 'Get single category');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Update category
                group('Update Category', () => {
                    const updateData = {
                        name: `Updated Category ${Date.now()}`,
                        description: 'This category has been updated',
                        sort_order: 10,
                        is_active: true
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/categories/${rootCategoryId}`,
                        JSON.stringify(updateData),
                        getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'update' })
                    );
                    
                    checkResponse(response, 200, 'Update category');
                });
            }
        });
        
        // Category hierarchy operations
        if (rootCategoryId) {
            group('Category Hierarchy', () => {
                // Create child category
                group('Create Child Category', () => {
                    const childCategory = createTestCategory();
                    childCategory.parent_id = rootCategoryId;
                    
                    const response = http.post(
                        `${API_BASE_URL}/api/categories`,
                        JSON.stringify(childCategory),
                        getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'create_child' })
                    );
                    
                    if (response.status === 201) {
                        checkResponse(response, 201, 'Create child category');
                        const created = parseResponse(response);
                        if (created && created.id) {
                            childCategoryId = created.id;
                            console.log(`Created child category: ${childCategoryId}`);
                        }
                    }
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Get children
                group('Get Category Children', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/categories/${rootCategoryId}/children`,
                        getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_children' })
                    );
                    
                    checkResponse(response, 200, 'Get category children');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Get category path
                if (childCategoryId) {
                    group('Get Category Path', () => {
                        const response = http.get(
                            `${API_BASE_URL}/api/categories/${childCategoryId}/path`,
                            getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_path' })
                        );
                        
                        checkResponse(response, 200, 'Get category path');
                    });
                    
                    sleep(sleepWithJitter(0.5));
                }
                
                // Move category
                if (childCategoryId) {
                    group('Move Category', () => {
                        // Create another parent category to move to
                        const anotherParent = createTestCategory();
                        const createResponse = http.post(
                            `${API_BASE_URL}/api/categories`,
                            JSON.stringify(anotherParent),
                            authHeaders
                        );
                        
                        if (createResponse.status === 201) {
                            const newParent = parseResponse(createResponse);
                            
                            const moveRequest = {
                                parent_id: newParent.id
                            };
                            
                            const response = http.put(
                                `${API_BASE_URL}/api/categories/${childCategoryId}/move`,
                                JSON.stringify(moveRequest),
                                getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'move' })
                            );
                            
                            checkResponse(response, 200, 'Move category');
                            
                            // Clean up the extra parent
                            http.del(`${API_BASE_URL}/api/categories/${newParent.id}`, null, authHeaders);
                        }
                    });
                }
            });
        }
        
        // Query with filters
        group('Category Queries', () => {
            // Get categories with parent filter
            if (rootCategoryId) {
                group('Get Categories by Parent', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/categories?parent_id=${rootCategoryId}`,
                        getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_by_parent' })
                    );
                    
                    checkResponse(response, 200, 'Get categories by parent');
                });
                
                sleep(sleepWithJitter(0.5));
            }
            
            // Get including inactive
            group('Get Including Inactive', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/categories?include_inactive=true`,
                    getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'get_with_inactive' })
                );
                
                checkResponse(response, 200, 'Get including inactive');
            });
        });
        
        // Cleanup
        group('Cleanup', () => {
            // Delete child category first
            if (childCategoryId) {
                const response = http.del(
                    `${API_BASE_URL}/api/categories/${childCategoryId}`,
                    null,
                    authHeaders
                );
                console.log(`Cleaned up child category ${childCategoryId}`);
            }
            
            sleep(0.5);
            
            // Delete root category
            if (rootCategoryId) {
                const response = http.del(
                    `${API_BASE_URL}/api/categories/${rootCategoryId}`,
                    null,
                    authHeaders
                );
                console.log(`Cleaned up root category ${rootCategoryId}`);
            }
        });
        
        // Error scenarios
        group('Error Scenarios', () => {
            // Try to create category with duplicate name
            group('Duplicate Name Error', () => {
                const category1 = createTestCategory();
                const response1 = http.post(
                    `${API_BASE_URL}/api/categories`,
                    JSON.stringify(category1),
                    authHeaders
                );
                
                if (response1.status === 201) {
                    const created = parseResponse(response1);
                    
                    // Try to create another with same name
                    const category2 = createTestCategory();
                    category2.name = category1.name;
                    
                    const response2 = http.post(
                        `${API_BASE_URL}/api/categories`,
                        JSON.stringify(category2),
                        getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'create_duplicate' })
                    );
                    
                    check(response2, {
                        'Duplicate name returns 409': (r) => r.status === 409
                    });
                    
                    // Cleanup
                    http.del(`${API_BASE_URL}/api/categories/${created.id}`, null, authHeaders);
                }
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Try to delete category with children
            group('Delete with Children Error', () => {
                const parent = createTestCategory();
                const parentResponse = http.post(
                    `${API_BASE_URL}/api/categories`,
                    JSON.stringify(parent),
                    authHeaders
                );
                
                if (parentResponse.status === 201) {
                    const parentCreated = parseResponse(parentResponse);
                    
                    const child = createTestCategory();
                    child.parent_id = parentCreated.id;
                    
                    const childResponse = http.post(
                        `${API_BASE_URL}/api/categories`,
                        JSON.stringify(child),
                        authHeaders
                    );
                    
                    if (childResponse.status === 201) {
                        const childCreated = parseResponse(childResponse);
                        
                        // Try to delete parent
                        const deleteResponse = http.del(
                            `${API_BASE_URL}/api/categories/${parentCreated.id}`,
                            null,
                            getAuthHeadersWithTags(token, { endpoint: 'categories', operation: 'delete_with_children' })
                        );
                        
                        check(deleteResponse, {
                            'Delete with children returns 400': (r) => r.status === 400
                        });
                        
                        // Cleanup
                        http.del(`${API_BASE_URL}/api/categories/${childCreated.id}`, null, authHeaders);
                        http.del(`${API_BASE_URL}/api/categories/${parentCreated.id}`, null, authHeaders);
                    }
                }
            });
        });
    });
    
    sleep(sleepWithJitter(2));
}