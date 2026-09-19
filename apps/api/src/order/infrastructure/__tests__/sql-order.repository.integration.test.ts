import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import { db } from '../../../database/db';
import { accounts, offers, orders, users } from '../../../database/schema';
import { SqlOrderRepository } from '../sql-order.repository';
import { randomUUID } from 'crypto';
import { eq, inArray } from 'drizzle-orm';

describe('SqlOrderRepository Integration', () => {
  let repository: SqlOrderRepository;
  
  let testAccountId1: string;
  let testUserId1: string; // offer owner
  
  let testAccountId2: string;
  let testUserId2: string; // requester
  
  let testOfferId1: string;
  let testOfferId2: string;

  beforeAll(async () => {
    repository = new SqlOrderRepository();
    
    testAccountId1 = randomUUID();
    testUserId1 = randomUUID();
    await db.insert(accounts).values({ id: testAccountId1, email: `order-owner-${Date.now()}@example.com` });
    await db.insert(users).values({ id: testUserId1, accountId: testAccountId1, username: `orderowner-${Date.now()}` });

    testAccountId2 = randomUUID();
    testUserId2 = randomUUID();
    await db.insert(accounts).values({ id: testAccountId2, email: `order-req-${Date.now()}@example.com` });
    await db.insert(users).values({ id: testUserId2, accountId: testAccountId2, username: `orderreq-${Date.now()}` });

    testOfferId1 = randomUUID();
    await db.insert(offers).values({
      id: testOfferId1,
      userId: testUserId1,
      title: 'Order Offer 1',
      price: 1000
    });
    
    testOfferId2 = randomUUID();
    await db.insert(offers).values({
      id: testOfferId2,
      userId: testUserId1,
      title: 'Order Offer 2',
      price: 2000
    });
  });

  afterAll(async () => {
    await db.delete(accounts).where(inArray(accounts.id, [testAccountId1, testAccountId2]));
  });

  it('should create an order and retrieve it by id', async () => {
    const orderId = randomUUID();
    const created = await repository.createOrder({
      id: orderId,
      offerId: testOfferId1,
      requesterId: testUserId2,
      price: 1000,
      quantity: 2,
      message: 'Hello'
    });
    
    expect(created.id).toBe(orderId);
    expect(created.status).toBe('PENDING');
    
    const fetched = await repository.getOrderById(orderId);
    expect(fetched).toBeDefined();
    expect(fetched?.price).toBe(1000);
    expect(fetched?.quantity).toBe(2);
    expect(fetched?.message).toBe('Hello');
  });

  it('should update order status', async () => {
    const orderId = randomUUID();
    await repository.createOrder({
      id: orderId,
      offerId: testOfferId1,
      requesterId: testUserId2,
      price: 1000,
      quantity: 1,
    });
    
    const updated = await repository.updateOrderStatus(orderId, 'ACCEPTED');
    expect(updated.status).toBe('ACCEPTED');
    
    const fetched = await repository.getOrderById(orderId);
    expect(fetched?.status).toBe('ACCEPTED');
  });

  it('should list orders by requester', async () => {
    const orderId = randomUUID();
    await repository.createOrder({
      id: orderId,
      offerId: testOfferId1,
      requesterId: testUserId2,
      price: 1000,
      quantity: 1,
    });
    
    const ordersList = await repository.getOrdersByRequester(testUserId2);
    expect(ordersList.length).toBeGreaterThan(0);
    const found = ordersList.find(o => o.id === orderId);
    expect(found).toBeDefined();
  });

  it('should list orders by offer owner', async () => {
    const orderId = randomUUID();
    await repository.createOrder({
      id: orderId,
      offerId: testOfferId2, // Owned by testUserId1
      requesterId: testUserId2,
      price: 2000,
      quantity: 1,
    });
    
    const receivedList = await repository.getOrdersByOfferOwner(testUserId1);
    expect(receivedList.length).toBeGreaterThan(0);
    const found = receivedList.find(o => o.id === orderId);
    expect(found).toBeDefined();
    expect(found?.offerId).toBe(testOfferId2);
  });
});
