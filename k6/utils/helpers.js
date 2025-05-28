import { check } from 'k6';

// Generate random data for testing
export function generateRandomString(length) {
    const charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += charset.charAt(Math.floor(Math.random() * charset.length));
    }
    return result;
}

export function generateRandomEmail() {
    return `test_${generateRandomString(10)}@example.com`;
}

export function generateRandomUsername() {
    return `user_${generateRandomString(8)}`;
}

export function generateRandomNumber(min, max) {
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

export function generateRandomPrice() {
    return parseFloat((Math.random() * 1000).toFixed(2));
}

// Check response status
export function checkResponse(response, expectedStatus, name) {
    const checks = {};
    checks[`${name} status is ${expectedStatus}`] = (r) => r.status === expectedStatus;
    checks[`${name} response time < 500ms`] = (r) => r.timings.duration < 500;
    
    if (expectedStatus >= 200 && expectedStatus < 300) {
        checks[`${name} has response body`] = (r) => r.body && r.body.length > 0;
    }
    
    return check(response, checks);
}

// Sleep helper with random jitter
export function sleepWithJitter(baseSeconds, jitterPercent = 0.2) {
    const jitter = baseSeconds * jitterPercent;
    const sleepTime = baseSeconds + (Math.random() * 2 - 1) * jitter;
    return Math.max(0.1, sleepTime); // Ensure at least 100ms sleep
}

// Parse response body safely
export function parseResponse(response) {
    try {
        return JSON.parse(response.body);
    } catch (e) {
        console.error(`Failed to parse response: ${e}`);
        return null;
    }
}

// Create test data generators
export function createTestItem() {
    return {
        name: `Test Item ${generateRandomString(8)}`,
        description: `This is a test item created at ${new Date().toISOString()}`
    };
}

export function createTestUser() {
    return {
        username: generateRandomUsername(),
        email: generateRandomEmail()
    };
}

export function createTestProduct() {
    return {
        name: `Product ${generateRandomString(8)}`,
        sku: `SKU-${generateRandomString(10).toUpperCase()}`,
        description: `Test product description ${generateRandomString(20)}`,
        price: generateRandomPrice(),
        stock_quantity: generateRandomNumber(0, 1000),
        category_id: null, // Will be set if categories exist
        is_active: true
    };
}

export function createTestCategory() {
    return {
        name: `Category ${generateRandomString(8)}`,
        description: `Test category ${generateRandomString(20)}`,
        parent_id: null,
        sort_order: generateRandomNumber(1, 100),
        is_active: true
    };
}