import { eq, inArray } from 'drizzle-orm';
import { db } from '../../database/db';
import { orders, offers, users } from '../../database/schema';
import { IOrderRepository } from '../domain/order.repository';
import { Order, OrderStatus } from '../domain/order.schema';

export class SqlOrderRepository implements IOrderRepository {
  async createOrder(data: { id: string; offerId: string; requesterId: string; price: number; quantity: number; message?: string | null; }): Promise<Order> {
    const [order] = await db.insert(orders).values({
      id: data.id,
      offerId: data.offerId,
      requesterId: data.requesterId,
      price: data.price,
      quantity: data.quantity,
      message: data.message,
    }).returning();
    return order;
  }

  async getOrderById(id: string): Promise<Order | null> {
    const [order] = await db.select().from(orders).where(eq(orders.id, id));
    return order || null;
  }

  async getOrdersByRequester(requesterId: string): Promise<Order[]> {
    return db.select().from(orders).where(eq(orders.requesterId, requesterId)).orderBy(orders.createdAt);
  }

  async getOrdersByOfferOwner(ownerId: string): Promise<Order[]> {
    // Join with offers to find orders where offer.userId === ownerId
    const result = await db.select({
      order: orders
    }).from(orders)
      .innerJoin(offers, eq(orders.offerId, offers.id))
      .where(eq(offers.userId, ownerId))
      .orderBy(orders.createdAt);
      
    return result.map(r => r.order);
  }

  async updateOrderStatus(id: string, status: OrderStatus): Promise<Order> {
    const [order] = await db.update(orders)
      .set({ status, updatedAt: new Date() })
      .where(eq(orders.id, id))
      .returning();
    return order;
  }
}
