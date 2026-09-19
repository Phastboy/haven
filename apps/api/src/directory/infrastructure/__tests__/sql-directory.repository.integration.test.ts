import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { db } from '../../../database/db';
import { users, accounts, offers } from '../../../database/schema';
import { SqlDirectoryRepository } from '../sql-directory.repository';
import { eq, inArray } from 'drizzle-orm';
import { randomUUID } from 'crypto';

describe('SqlDirectoryRepository Integration', () => {
  let repository: SqlDirectoryRepository;
  
  let testAccountId1: string;
  let testUserId1: string;
  let testAccountId2: string;
  let testUserId2: string;
  let activeOfferId1: string;
  let activeOfferId2: string;
  let pausedOfferId: string;

  beforeAll(async () => {
    repository = new SqlDirectoryRepository();
    
    testAccountId1 = randomUUID();
    testUserId1 = randomUUID();
    await db.insert(accounts).values({ id: testAccountId1, email: `dir1-${Date.now()}@example.com` });
    await db.insert(users).values({ id: testUserId1, accountId: testAccountId1, username: `dir1-${Date.now()}` });

    testAccountId2 = randomUUID();
    testUserId2 = randomUUID();
    await db.insert(accounts).values({ id: testAccountId2, email: `dir2-${Date.now()}@example.com` });
    await db.insert(users).values({ id: testUserId2, accountId: testAccountId2, username: `dir2-${Date.now()}` });

    activeOfferId1 = randomUUID();
    await db.insert(offers).values({
      id: activeOfferId1,
      userId: testUserId1,
      title: 'Active Offer 1',
      status: 'ACTIVE',
      price: 1000,
      offerType: 'PRODUCT'
    });

    activeOfferId2 = randomUUID();
    await db.insert(offers).values({
      id: activeOfferId2,
      userId: testUserId2,
      title: 'Active Offer 2',
      status: 'ACTIVE',
      price: 2000,
      offerType: 'SERVICE'
    });

    pausedOfferId = randomUUID();
    await db.insert(offers).values({
      id: pausedOfferId,
      userId: testUserId1,
      title: 'Paused Offer',
      status: 'PAUSED',
      price: 500,
      offerType: 'PRODUCT'
    });
  });

  afterAll(async () => {
    await db.delete(accounts).where(inArray(accounts.id, [testAccountId1, testAccountId2]));
  });

  it('should list only active offers sorted by date', async () => {
    const activeOffers = await repository.getActiveOffers(10, 0);
    expect(activeOffers.length).toBeGreaterThanOrEqual(2);
    
    const titles = activeOffers.map(o => o.title);
    expect(titles).toContain('Active Offer 1');
    expect(titles).toContain('Active Offer 2');
    expect(titles).not.toContain('Paused Offer');
  });

  it('should batch fetch users by ids', async () => {
    const fetchedUsers = await repository.getUsersByIds([testUserId1, testUserId2]);
    expect(fetchedUsers.length).toBe(2);
    
    const fetchedIds = fetchedUsers.map(u => u.id);
    expect(fetchedIds).toContain(testUserId1);
    expect(fetchedIds).toContain(testUserId2);
  });
});
