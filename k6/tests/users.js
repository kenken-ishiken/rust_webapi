import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { API_BASE_URL } from '../config/constants.js';
import { getAccessToken, getAuthHeaders, getAuthHeadersWithTags } from '../utils/auth.js';
import { checkResponse, createTestUser, sleepWithJitter, parseResponse } from '../utils/helpers.js';

// Test configuration
export let options = {
    thresholds: {
        'http_req_duration{endpoint:users}': ['p(95)<500', 'p(99)<1000'],
        'http_req_failed{endpoint:users}': ['rate<0.05'],
        'group_duration{group:User CRUD}': ['p(95)<2000'],
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
    
    group('User API Tests', () => {
        let createdUserId;
        
        group('User CRUD', () => {
            // Get all users
            group('Get All Users', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/users`,
                    getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'get_all' })
                );
                
                checkResponse(response, 200, 'Get all users');
                
                const users = parseResponse(response);
                if (users && Array.isArray(users)) {
                    console.log(`Found ${users.length} users`);
                }
            });
            
            sleep(sleepWithJitter(1));
            
            // Create user
            group('Create User', () => {
                const newUser = createTestUser();
                
                const response = http.post(
                    `${API_BASE_URL}/api/users`,
                    JSON.stringify(newUser),
                    getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'create' })
                );
                
                checkResponse(response, 201, 'Create user');
                
                const created = parseResponse(response);
                if (created && created.id) {
                    createdUserId = created.id;
                    console.log(`Created user: ${createdUserId} (${created.username})`);
                }
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Get single user
            if (createdUserId) {
                group('Get Single User', () => {
                    const response = http.get(
                        `${API_BASE_URL}/api/users/${createdUserId}`,
                        getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'get_one' })
                    );
                    
                    checkResponse(response, 200, 'Get single user');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Update user
                group('Update User', () => {
                    const updateData = {
                        username: `updated_user_${Date.now()}`,
                        email: `updated_${Date.now()}@example.com`
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/users/${createdUserId}`,
                        JSON.stringify(updateData),
                        getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'update' })
                    );
                    
                    checkResponse(response, 200, 'Update user');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Partial update user
                group('Partial Update User', () => {
                    const partialUpdateData = {
                        email: `partial_update_${Date.now()}@example.com`
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/users/${createdUserId}`,
                        JSON.stringify(partialUpdateData),
                        getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'partial_update' })
                    );
                    
                    checkResponse(response, 200, 'Partial update user');
                });
                
                sleep(sleepWithJitter(0.5));
                
                // Delete user
                group('Delete User', () => {
                    const response = http.del(
                        `${API_BASE_URL}/api/users/${createdUserId}`,
                        null,
                        getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'delete' })
                    );
                    
                    checkResponse(response, 200, 'Delete user');
                });
            }
        });
        
        // Error scenarios
        group('Error Scenarios', () => {
            // Get non-existent user
            group('Get Non-existent User', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/users/999999`,
                    getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'get_not_found' })
                );
                
                check(response, {
                    'Get non-existent user returns 404': (r) => r.status === 404
                });
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Update non-existent user
            group('Update Non-existent User', () => {
                const updateData = {
                    username: 'ghost_user',
                    email: 'ghost@example.com'
                };
                
                const response = http.put(
                    `${API_BASE_URL}/api/users/999999`,
                    JSON.stringify(updateData),
                    getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'update_not_found' })
                );
                
                check(response, {
                    'Update non-existent user returns 404': (r) => r.status === 404
                });
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Delete non-existent user
            group('Delete Non-existent User', () => {
                const response = http.del(
                    `${API_BASE_URL}/api/users/999999`,
                    null,
                    getAuthHeadersWithTags(token, { endpoint: 'users', operation: 'delete_not_found' })
                );
                
                check(response, {
                    'Delete non-existent user returns 404': (r) => r.status === 404
                });
            });
        });
        
        // Bulk operations simulation
        group('Bulk User Operations', () => {
            const userIds = [];
            
            // Create multiple users
            group('Bulk Create Users', () => {
                for (let i = 0; i < 5; i++) {
                    const user = createTestUser();
                    const response = http.post(
                        `${API_BASE_URL}/api/users`,
                        JSON.stringify(user),
                        authHeaders
                    );
                    
                    if (response.status === 201) {
                        const created = parseResponse(response);
                        userIds.push(created.id);
                    }
                    sleep(0.1); // Small delay between creates
                }
                
                console.log(`Created ${userIds.length} users for bulk testing`);
            });
            
            sleep(sleepWithJitter(1));
            
            // Read all created users
            group('Bulk Read Users', () => {
                userIds.forEach((userId, index) => {
                    const response = http.get(
                        `${API_BASE_URL}/api/users/${userId}`,
                        authHeaders
                    );
                    
                    if (index === 0) { // Only check first one to avoid too many checks
                        checkResponse(response, 200, 'Bulk read user');
                    }
                    sleep(0.05);
                });
            });
            
            sleep(sleepWithJitter(1));
            
            // Update all created users
            group('Bulk Update Users', () => {
                userIds.forEach((userId, index) => {
                    const updateData = {
                        username: `bulk_updated_${userId}_${Date.now()}`,
                        email: `bulk_${userId}@example.com`
                    };
                    
                    const response = http.put(
                        `${API_BASE_URL}/api/users/${userId}`,
                        JSON.stringify(updateData),
                        authHeaders
                    );
                    
                    if (index === 0) { // Only check first one
                        checkResponse(response, 200, 'Bulk update user');
                    }
                    sleep(0.05);
                });
            });
            
            sleep(sleepWithJitter(1));
            
            // Delete all created users
            group('Bulk Delete Users', () => {
                userIds.forEach((userId, index) => {
                    const response = http.del(
                        `${API_BASE_URL}/api/users/${userId}`,
                        null,
                        authHeaders
                    );
                    
                    if (index === 0) { // Only check first one
                        checkResponse(response, 200, 'Bulk delete user');
                    }
                    sleep(0.05);
                });
                
                console.log(`Deleted ${userIds.length} users`);
            });
        });
        
        // Authentication scenarios
        group('Authentication Tests', () => {
            // Test without token
            group('No Authentication', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/users`,
                    { tags: { endpoint: 'users', operation: 'no_auth' } }
                );
                
                check(response, {
                    'No auth returns 401 or 403': (r) => r.status === 401 || r.status === 403
                });
            });
            
            sleep(sleepWithJitter(0.5));
            
            // Test with invalid token
            group('Invalid Token', () => {
                const response = http.get(
                    `${API_BASE_URL}/api/users`,
                    {
                        headers: {
                            'Authorization': 'Bearer invalid_token_12345',
                            'Content-Type': 'application/json'
                        },
                        tags: { endpoint: 'users', operation: 'invalid_auth' }
                    }
                );
                
                check(response, {
                    'Invalid token returns 401': (r) => r.status === 401
                });
            });
        });
    });
    
    sleep(sleepWithJitter(2));
}