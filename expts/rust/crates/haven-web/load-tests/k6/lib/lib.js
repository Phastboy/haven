import { SharedArray } from 'k6/data';
import http from 'k6/http';
import { check } from 'k6';

// Read the pre-generated users from JSON file
export const users = new SharedArray('users', function () {
    return JSON.parse(open('../../data/sessions.json'));
});

// Create a deterministic session mapping for the VU
export function getSession(vuId, iteration, maxVUs) {
    const userIndex = vuId - 1; 
    // vuId is 1-indexed. If there are fewer users than VUs, we wrap around.
    return users[userIndex % users.length];
}

export function extractIdempotencyKey(html) {
    const match = html.match(/idempotency_key["']?[^>]+value=["']?([^"'\s>]+)/);
    if (!match) {
        const match2 = html.match(/value=["']?([^"'\s>]+)["']?[^>]+name=["']?idempotency_key["']?/);
        return match2 ? match2[1] : null;
    }
    return match[1];
}

// Basic threshold definitions
export const defaultThresholds = {
    http_req_failed: ['rate==0.0'], // 0% errors
    http_req_duration: ['p(95)<=200', 'p(99)<=500'], 
};

// Generate headers for an authenticated user
export function getHeaders(sessionToken, clientIp) {
    const encodedToken = sessionToken.replace('=', '%3D');
    const headers = {
        'Cookie': `__Host-sid=${encodedToken}`,
    };
    if (clientIp) {
        // Mock a real client IP if rate limiting is enabled
        headers['X-Forwarded-For'] = clientIp;
    }
    return headers;
}
