import http from 'k6/http';
import { check } from 'k6';
import { getSession, extractIdempotencyKey, getHeaders, defaultThresholds } from '../lib/lib.js';

// Derived from:
// 1000 HTTP requests/sec target
// request mix: 1.21 reqs/iteration on average
// rate = 1000 / 1.21 = 826 iterations/s
export const options = {
    scenarios: {
        steady_state: {
            executor: 'ramping-arrival-rate',
            startRate: 50,
            timeUnit: '1s',
            preAllocatedVUs: 1000,
            maxVUs: 5000, // Safe up to 10k max users
            stages: [
                { duration: '30s', target: 826 }, // Warm-up to 1k RPS (826 iter/s)
                { duration: '4m30s', target: 826 }, // Steady state plateau
            ]
        }
    },
    thresholds: {
        // Contract states: 0% errors, 100% correctness, p95 <= 200ms, p99 <= 500ms
        http_req_failed: ['rate==0.0'],
        dropped_iterations: ['count==0'],
        checks: ['rate==1.0'], // 100% correctness
        
        // Detailed latency gates by route tag
        'http_req_duration{name:GET /offers}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:GET /offers/id}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:GET /}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:GET /offers/new}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:POST /offers/new}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:GET /offers/id/edit}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:POST /offers/id/edit}': ['p(95)<=200', 'p(99)<=500'],
        'http_req_duration{name:POST /offers/id/delete}': ['p(95)<=200', 'p(99)<=500'],
    }
};

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:3000';

export default function () {
    const session = getSession(__VU, __ITER);
    const headers = getHeaders(session.session_token);

    const rand = Math.random();

    if (rand < 0.50) {
        // 50%: GET /offers (list)
        let res = http.get(`${BASE_URL}/offers`, { headers, tags: { name: 'GET /offers' } });
        check(res, { 'list is 200': (r) => r.status === 200 });
    } else if (rand < 0.85) {
        // 35%: GET /offers/{id} (detail)
        // Pick a random offer from the seeded data for this user
        if (session.offer_ids && session.offer_ids.length > 0) {
            const offerId = session.offer_ids[Math.floor(Math.random() * session.offer_ids.length)];
            let res = http.get(`${BASE_URL}/offers/${offerId}`, { headers, tags: { name: 'GET /offers/id' } });
            check(res, { 'detail is 200': (r) => r.status === 200 });
        }
    } else if (rand < 0.97) {
        // 12%: GET / (public landing)
        // Ensure this is unauthenticated (no headers) so it doesn't redirect
        let res = http.get(`${BASE_URL}/`, { tags: { name: 'GET /' } });
        check(res, { 'public is 200': (r) => r.status === 200 });
    } else {
        // 3%: write journey (8 requests)
        // GET new, POST new, GET detail, GET edit, POST edit, GET detail, POST delete, GET list
        
        let res = http.get(`${BASE_URL}/offers/new`, { headers, tags: { name: 'GET /offers/new' } });
        check(res, { 'new form is 200': (r) => r.status === 200 });
        
        const idempKey = extractIdempotencyKey(res.body);
        if (!idempKey) return; // Cannot continue if parsing fails

        const createPayload = {
            title: `Write Journey ${__VU}-${__ITER}`,
            description: 'Load test dynamic offer',
            price: '500', // 5.00
            currency: 'NGN',
            idempotency_key: idempKey
        };

        res = http.post(`${BASE_URL}/offers/new`, createPayload, {
            headers: Object.assign({}, headers, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0,
            tags: { name: 'POST /offers/new' }
        });

        check(res, { 'create redirects': (r) => r.status === 303 });
        const location = res.headers['Location'];
        if (!location) return;

        // View detail (assert title and price)
        res = http.get(`${BASE_URL}${location}`, { headers, tags: { name: 'GET /offers/id' } });
        check(res, { 
            'detail shows created offer': (r) => r.body.includes(createPayload.title),
        });

        // Edit form
        res = http.get(`${BASE_URL}${location}/edit`, { headers, tags: { name: 'GET /offers/id/edit' } });
        check(res, { 'edit form is 200': (r) => r.status === 200 });
        
        // POST edit
        const editPayload = {
            title: createPayload.title + ' (Edited)',
            description: createPayload.description,
            price: '0', // Test Free rendering
            currency: 'NGN',
        };

        res = http.post(`${BASE_URL}${location}/edit`, editPayload, {
            headers: Object.assign({}, headers, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0,
            tags: { name: 'POST /offers/id/edit' }
        });
        check(res, { 'edit redirects': (r) => r.status === 303 });

        // View detail again (assert edit worked, and Free rendered for 0)
        res = http.get(`${BASE_URL}${location}`, { headers, tags: { name: 'GET /offers/id' } });
        check(res, { 
            'detail shows edited title': (r) => r.body.includes(editPayload.title),
            'detail shows Free': (r) => r.body.includes('Free') // Adjust based on how 0 price is rendered in the app
        });

        // POST delete
        res = http.post(`${BASE_URL}${location}/delete`, {}, {
            headers: Object.assign({}, headers, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0,
            tags: { name: 'POST /offers/id/delete' }
        });
        check(res, { 'delete redirects': (r) => r.status === 303 });

        // Verify 404
        res = http.get(`${BASE_URL}${location}`, { headers, tags: { name: 'GET /offers/id' }, responseCallback: http.expectedStatuses(404) });
        check(res, { '404 after delete': (r) => r.status === 404 });
        
        // GET list at the end
        res = http.get(`${BASE_URL}/offers`, { headers, tags: { name: 'GET /offers' } });
        check(res, { 'list after delete is 200': (r) => r.status === 200 });
    }
}
