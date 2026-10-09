import http from 'k6/http';
import { check } from 'k6';
import { getSession, extractIdempotencyKey, getHeaders } from '../lib/lib.js';

// Mix is 1.21 reqs/iter. 
// Capacity test targets breaking point (3000+ RPS). 
// So 3000 RPS is ~2479 iter/s.
export const options = {
    scenarios: {
        capacity: {
            executor: 'ramping-arrival-rate',
            startRate: 500,
            timeUnit: '1s',
            preAllocatedVUs: 1000,
            maxVUs: 10000,
            stages: [
                { duration: '1m', target: 826 }, // Warm up to 1k RPS (826 iter/s)
                { duration: '2m', target: 826 }, // Plateau at 1k RPS
                { duration: '2m', target: 1652 }, // Step up to 2k RPS (1652 iter/s)
                { duration: '2m', target: 1652 }, // Plateau at 2k RPS
                { duration: '2m', target: 2479 }, // Step up to 3k RPS (2479 iter/s)
                { duration: '2m', target: 2479 }, // Plateau at 3k RPS
                { duration: '2m', target: 3305 }, // Step up to 4k RPS
                { duration: '2m', target: 3305 }, // Plateau at 4k RPS
            ]
        }
    },
    thresholds: {
        // Abort on fail if errors exceed 1% or p99 > 1s during the capacity run
        http_req_failed: [{ threshold: 'rate<=0.01', abortOnFail: true }],
        http_req_duration: [{ threshold: 'p(99)<=1000', abortOnFail: true }],
    }
};

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:3000';

export default function () {
    const session = getSession(__VU, __ITER);
    const headers = getHeaders(session.session_token);

    const rand = Math.random();

    if (rand < 0.50) {
        http.get(`${BASE_URL}/offers`, { headers, tags: { name: 'GET /offers' } });
    } else if (rand < 0.85) {
        if (session.offer_ids && session.offer_ids.length > 0) {
            const offerId = session.offer_ids[Math.floor(Math.random() * session.offer_ids.length)];
            http.get(`${BASE_URL}/offers/manage/${offerId}`, { headers, tags: { name: 'GET /offers/id' } });
        }
    } else if (rand < 0.97) {
        http.get(`${BASE_URL}/`, { tags: { name: 'GET /' } });
    } else {
        let res = http.get(`${BASE_URL}/offers/manage/new`, { headers, tags: { name: 'GET /offers/new' } });
        const idempKey = extractIdempotencyKey(res.body);
        check(idempKey, { 'extracted idempotency key': (k) => !!k });
        if (!idempKey) return;

        const createPayload = {
            title: `Capacity Write ${__VU}-${__ITER}`,
            price: '500',
            currency: 'NGN',
            idempotency_key: idempKey
        };

        res = http.post(`${BASE_URL}/offers/manage/new`, createPayload, {
            headers: Object.assign({}, headers, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0,
            tags: { name: 'POST /offers/new' }
        });

        check(res, { 'create redirects': (r) => r.status === 303 });

        const location = res.headers['Location'];
        check(location, { 'location header present': (l) => !!l });
        if (!location) return;

        res = http.get(`${BASE_URL}${location}`, { headers, tags: { name: 'GET /offers/id' } });
        res = http.get(`${BASE_URL}${location}/edit`, { headers, tags: { name: 'GET /offers/id/edit' } });
        
        const editPayload = {
            title: createPayload.title + ' (Edited)',
            price: '0',
            currency: 'NGN',
        };

        res = http.post(`${BASE_URL}${location}/edit`, editPayload, {
            headers: Object.assign({}, headers, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0,
            tags: { name: 'POST /offers/id/edit' }
        });

        res = http.get(`${BASE_URL}${location}`, { headers, tags: { name: 'GET /offers/id' } });
        
        res = http.post(`${BASE_URL}${location}/delete`, {}, {
            headers: Object.assign({}, headers, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0,
            tags: { name: 'POST /offers/id/delete' }
        });

        res = http.get(`${BASE_URL}${location}`, { headers, tags: { name: 'GET /offers/id' }, responseCallback: http.expectedStatuses(404) });
        res = http.get(`${BASE_URL}/offers`, { headers, tags: { name: 'GET /offers' } });
    }
}
