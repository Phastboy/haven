import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { randomUUID } from 'crypto';
import { db } from '../../../database/db';
import { users, accounts, offers } from '../../../database/schema';
import { app } from '../../../index';
import { eq } from 'drizzle-orm';

describe('GraphQL Directory E2E', () => {
  let testAccountId1: string;
  let testUserId1: string;
  let activeOfferId1: string;

  beforeAll(async () => {
    testAccountId1 = randomUUID();
    testUserId1 = randomUUID();
    
    await db.insert(accounts).values({ id: testAccountId1, email: `graphel-e2e-${Date.now()}@example.com` });
    await db.insert(users).values({ id: testUserId1, accountId: testAccountId1, username: `graphqle2e-${Date.now()}` });

    activeOfferId1 = randomUUID();
    await db.insert(offers).values({
      id: activeOfferId1,
      userId: testUserId1,
      title: 'GraphQL E2E Active Offer',
      status: 'ACTIVE',
      price: 3500,
      offerType: 'SERVICE'
    });
  });

  afterAll(async () => {
    // Cascade delete on account
    await db.delete(accounts).where(eq(accounts.id, testAccountId1));
  });

  it('should resolve activeOffers with nested user data via GraphQL', async () => {
    const query = `
      query {
        activeOffers(limit: 5) {
          id
          title
          price
          user {
            id
            username
          }
        }
      }
    `;

    const req = new Request('http://localhost/api/graphql', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ query }),
    });

    const res = await app.handle(req);
    expect(res.status).toBe(200);

    const body: any = await res.json();
    expect(body.errors).toBeUndefined();
    expect(body.data).toBeDefined();
    
    const fetchedOffers = body.data.activeOffers;
    expect(Array.isArray(fetchedOffers)).toBe(true);
    expect(fetchedOffers.length).toBeGreaterThan(0);

    const testOffer = fetchedOffers.find((o: any) => o.id === activeOfferId1);
    expect(testOffer).toBeDefined();
    expect(testOffer.title).toBe('GraphQL E2E Active Offer');
    expect(testOffer.price).toBe(3500);
    expect(testOffer.user).toBeDefined();
    expect(testOffer.user.id).toBe(testUserId1);
    expect(testOffer.user.username.startsWith('graphqle2e-')).toBe(true);
  });
});
