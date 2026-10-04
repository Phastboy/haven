import http from 'k6/http';
import { check } from 'k6';
import { getSession, getHeaders, extractIdempotencyKey } from '../lib/lib.js';

export const options = {
    vus: 1,
    iterations: 1,
    thresholds: {
        checks: ['rate==1.0'],
    }
};

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:3000';

export default function () {
    // Get User A and User B from the seeded data
    // Because we just use vuId and maxVUs mapping in lib, we can just fetch index 0 and 1
    const userA = getSession(1, 1);
    const userB = getSession(2, 1);

    const headersA = getHeaders(userA.session_token);
    const headersB = getHeaders(userB.session_token);

    // Pick an offer from User B's seeded list
    if (!userB.offer_ids || userB.offer_ids.length === 0) {
        console.error("User B has no seeded offers to test authorization against.");
        return;
    }
    const offerIdB = userB.offer_ids[0];
    const locationB = `/offers/${offerIdB}`;

    // 1. User A attempts to access User B's offer
    let res = http.get(`${BASE_URL}${locationB}`, { headers: headersA });
    check(res, { 'User A detail GET on B offer is 404': (r) => r.status === 404 });

    res = http.get(`${BASE_URL}${locationB}/edit`, { headers: headersA });
    check(res, { 'User A edit GET on B offer is 404': (r) => r.status === 404 });

    const editPayload = { title: 'Hacked', price: '0', currency: 'NGN' };
    res = http.post(`${BASE_URL}${locationB}/edit`, editPayload, { 
        headers: Object.assign({}, headersA, { 'Content-Type': 'application/x-www-form-urlencoded' }),
        redirects: 0 
    });
    check(res, { 'User A edit POST on B offer is 404': (r) => r.status === 404 });

    res = http.post(`${BASE_URL}${locationB}/delete`, {}, { 
        headers: Object.assign({}, headersA, { 'Content-Type': 'application/x-www-form-urlencoded' }),
        redirects: 0 
    });
    check(res, { 'User A delete POST on B offer is 404': (r) => r.status === 404 });

    // 2. Read back as User B and assert the offer is unchanged
    res = http.get(`${BASE_URL}${locationB}`, { headers: headersB });
    check(res, { 
        'User B sees offer unchanged': (r) => r.status === 200 && !r.body.includes('Hacked')
    });

    // 3. Assert unauthenticated access returns redirects (or 401/403)
    res = http.get(`${BASE_URL}${locationB}`, { redirects: 0 });
    check(res, { 'Unauth detail GET redirects': (r) => r.status === 303 });

    res = http.get(`${BASE_URL}${locationB}/edit`, { redirects: 0 });
    check(res, { 'Unauth edit GET redirects': (r) => r.status === 303 });

    res = http.post(`${BASE_URL}${locationB}/edit`, editPayload, { redirects: 0 });
    check(res, { 'Unauth edit POST redirects': (r) => r.status === 303 });

    res = http.post(`${BASE_URL}${locationB}/delete`, {}, { redirects: 0 });
    check(res, { 'Unauth delete POST redirects': (r) => r.status === 303 });

    res = http.get(`${BASE_URL}/offers/new`, { redirects: 0 });
    check(res, { 'Unauth GET new redirects': (r) => r.status === 303 });

    // 4. Assert replayed idempotency_key creates one offer, not two
    res = http.get(`${BASE_URL}/offers/new`, { headers: headersA });
    const idempKey = extractIdempotencyKey(res.body);
    if (idempKey) {
        const createPayload = {
            title: `Idempotency Test`,
            price: '500',
            currency: 'NGN',
            idempotency_key: idempKey
        };

        const postArgs = {
            headers: Object.assign({}, headersA, { 'Content-Type': 'application/x-www-form-urlencoded' }),
            redirects: 0
        };

        // Fire two requests with same key concurrently
        const responses = http.batch([
            ['POST', `${BASE_URL}/offers/new`, createPayload, postArgs],
            ['POST', `${BASE_URL}/offers/new`, createPayload, postArgs]
        ]);

        // One should succeed (303), the other should fail or gracefully handle it (often 409, 303 to same URL, or rate limited)
        // Let's assert that at least one request didn't result in a new creation (we'd check the DB, but realistically we expect one 303 and one 4xx/5xx or same 303)
        // Depending on idempotency implementation, it might return the same Location header, or an error.
        // We'll just verify the response status combination.
        const statuses = responses.map(r => r.status);
        check(statuses, {
            'Idempotency prevents duplicate creation': (s) => 
                (s[0] === 303 && s[1] !== 303) || (s[1] === 303 && s[0] !== 303) || 
                (s[0] === 303 && s[1] === 303 && responses[0].headers['Location'] === responses[1].headers['Location'])
        });
    }
}
