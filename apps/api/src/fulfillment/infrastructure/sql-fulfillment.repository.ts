import { eq, lt, and } from 'drizzle-orm';
import { db } from '../../database/db';
import { fulfillments } from '../../database/schema';
import { IFulfillmentRepository } from '../domain/fulfillment.repository';
import { Fulfillment, FulfillmentStatus } from '../domain/fulfillment.schema';

export class SqlFulfillmentRepository implements IFulfillmentRepository {
  async createFulfillment(data: { id: string; orderId: string }): Promise<Fulfillment> {
    const [fulfillment] = await db.insert(fulfillments).values({
      id: data.id,
      orderId: data.orderId,
    }).returning();
    return fulfillment;
  }

  async getFulfillmentById(id: string): Promise<Fulfillment | null> {
    const [fulfillment] = await db.select().from(fulfillments).where(eq(fulfillments.id, id));
    return fulfillment || null;
  }

  async getFulfillmentByOrderId(orderId: string): Promise<Fulfillment | null> {
    const [fulfillment] = await db.select().from(fulfillments).where(eq(fulfillments.orderId, orderId));
    return fulfillment || null;
  }

  async updateFulfillmentStatus(
    id: string,
    status: FulfillmentStatus,
    options?: { deliveryMessage?: string; reviewDeadline?: Date }
  ): Promise<Fulfillment> {
    const [fulfillment] = await db.update(fulfillments)
      .set({
        status,
        ...(options?.deliveryMessage !== undefined && { deliveryMessage: options.deliveryMessage }),
        ...(options?.reviewDeadline !== undefined && { reviewDeadline: options.reviewDeadline }),
        updatedAt: new Date(),
      })
      .where(eq(fulfillments.id, id))
      .returning();
    return fulfillment;
  }

  async getExpiredFulfillments(currentDate: Date): Promise<Fulfillment[]> {
    return db.select()
      .from(fulfillments)
      .where(
        and(
          eq(fulfillments.status, 'DELIVERED'),
          lt(fulfillments.reviewDeadline, currentDate)
        )
      );
  }
}
