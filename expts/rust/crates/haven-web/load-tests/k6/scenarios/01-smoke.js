import http from 'k6/http';
import { check } from 'k6';
import { getSession, extractIdempotencyKey, getHeaders } from '../lib/lib.js';

export const options = {
    vus: 1,
    iterations: 1,
    thresholds: {
        checks: ['rate==1.0'],
    }
};

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:3000';

export default function () {
    const session = getSession(1, 1, 1);
    const headers = getHeaders(session.session_token);

    // 1. Visit new offer page
    let res = http.get(`${BASE_URL}/offers/new`, { headers });
    check(res, {
        'GET /offers/new is 200': (r) => r.status === 200,
        'has idempotency key': (r) => extractIdempotencyKey(r.body) !== null,
    });

    const idempKey = extractIdempotencyKey(res.body);

    // 2. Create offer
    // Use manual redirects so we can inspect the location header
    const payload = {
        title: 'Smoke Test Offer',
        description: 'Smoke test description',
        price: '5000',
        currency: 'NGN',
        idempotency_key: idempKey
    };

    res = http.post(`${BASE_URL}/offers/new`, payload, {
        headers: Object.assign({}, headers, {
            'Content-Type': 'application/x-www-form-urlencoded'
        }),
        redirects: 0
    });

    check(res, {
        'POST /offers/new is 303': (r) => r.status === 303,
        'has Location header': (r) => r.headers['Location'] !== undefined
    });

    const location = res.headers['Location'];

    // 3. View the created offer
    res = http.get(`${BASE_URL}${location}`, { headers });
    check(res, {
        'GET /offers/{id} is 200': (r) => r.status === 200,
        'shows correct title': (r) => r.body.includes('Smoke Test Offer')
    });

    // 4. Delete the offer
    // Actually, Haven uses `POST /offers/{id}/delete` for deletions in HTML
    res = http.post(`${BASE_URL}${location}/delete`, {}, {
        headers: Object.assign({}, headers, {
            'Content-Type': 'application/x-www-form-urlencoded'
        }),
        redirects: 0
    });

    check(res, {
        'POST delete is 303': (r) => r.status === 303
    });

    // 5. Verify the offer is gone
    res = http.get(`${BASE_URL}${location}`, { headers, responseCallback: http.expectedStatuses(404) });
    check(res, {
        'GET /offers/{id} is 404 after delete': (r) => r.status === 404
    });
}
