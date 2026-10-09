import http from 'k6/http';
import { check, fail } from 'k6';
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

    check(userA.user_id !== userB.user_id, { 'users are distinct': (v) => v });
    if (userA.user_id === userB.user_id) fail("Not enough seeded users for distinct roles");

    const headersA = getHeaders(userA.session_token);
    const headersB = getHeaders(userB.session_token);

    check(userB, { 'User B has offers': (u) => u.offer_ids && u.offer_ids.length > 0 });
    if (!userB.offer_ids || userB.offer_ids.length === 0) {
        fail("User B has no seeded offers to test authorization against.");
    }
    const offerIdB = userB.offer_ids[0];
    const locationB = `/offers/manage/${offerIdB}`;

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

    res = http.get(`${BASE_URL}/offers/manage/new`, { redirects: 0 });
    check(res, { 'Unauth GET new redirects': (r) => r.status === 303 });

    // 4. Assert replayed idempotency_key creates one offer, not two
    res = http.get(`${BASE_URL}/offers/manage/new`, { headers: headersA });
    const idempKey = extractIdempotencyKey(res.body);
    
    check(idempKey, { 'extracted idempotency key': (k) => !!k });
    if (!idempKey) fail("Missing idempotency key in /offers/manage/new response");

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
        ['POST', `${BASE_URL}/offers/manage/new`, createPayload, postArgs],
        ['POST', `${BASE_URL}/offers/manage/new`, createPayload, postArgs]
    ]);

    const statuses = responses.map(r => r.status);
    check(statuses, {
        'Idempotency prevents duplicate creation': (s) => 
            (s[0] === 303 && s[1] !== 303) || (s[1] === 303 && s[0] !== 303) || 
            (s[0] === 303 && s[1] === 303 && responses[0].headers['Location'] === responses[1].headers['Location'])
    });
}
