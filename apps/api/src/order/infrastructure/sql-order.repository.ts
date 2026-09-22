import { and, eq } from "drizzle-orm";
import { db } from "../../database/db";
import { orders, offers } from "../../database/schema";
import { IOrderRepository } from "../domain/order.repository";
import { Order, OrderStatus } from "../domain/order.schema";
import { InvalidOrderStateTransitionError } from "../domain/errors";

export class SqlOrderRepository implements IOrderRepository {
  async createOrder(data: {
    id: string;
    offerId: string;
    requesterId: string;
    price: number;
    quantity: number;
    message?: string | null;
  }): Promise<Order> {
    const [order] = await db
      .insert(orders)
      .values({
        id: data.id,
        offerId: data.offerId,
        requesterId: data.requesterId,
        price: data.price,
        quantity: data.quantity,
        message: data.message,
      })
      .returning();
    return order!;
  }

  async getOrderById(id: string): Promise<Order | null> {
    const [order] = await db.select().from(orders).where(eq(orders.id, id));
    return order || null;
  }

  async getOrdersByRequester(requesterId: string): Promise<Order[]> {
    return db
      .select()
      .from(orders)
      .where(eq(orders.requesterId, requesterId))
      .orderBy(orders.createdAt);
  }

  async getOrdersByOfferOwner(ownerId: string): Promise<Order[]> {
    // Join with offers to find orders where offer.userId === ownerId
    const result = await db
      .select({
        order: orders,
      })
      .from(orders)
      .innerJoin(offers, eq(orders.offerId, offers.id))
      .where(eq(offers.userId, ownerId))
      .orderBy(orders.createdAt);

    return result.map((r) => r.order);
  }

  async updateOrderStatus(id: string, status: OrderStatus): Promise<Order> {
    // Atomic conditional update: only succeeds if the order is currently PENDING.
    // This prevents a race condition where two concurrent ACCEPT requests both
    // read status=PENDING before either write completes.
    const [order] = await db
      .update(orders)
      .set({ status, updatedAt: new Date() })
      .where(and(eq(orders.id, id), eq(orders.status, "PENDING")))
      .returning();

    if (!order) {
      // Race was lost — fetch the current state and surface a proper domain error.
      const current = await this.getOrderById(id);
      if (!current) {
        // Should not happen (use-case already validated the order exists), but be safe.
        throw new InvalidOrderStateTransitionError("Order not found.");
      }
      throw new InvalidOrderStateTransitionError(
        `Cannot transition from ${current.status} to ${status}: order is no longer PENDING.`,
      );
    }

    return order;
  }

  async findPendingByRequesterAndOffer(
    requesterId: string,
    offerId: string,
  ): Promise<Order | null> {
    const [order] = await db
      .select()
      .from(orders)
      .where(
        and(
          eq(orders.requesterId, requesterId),
          eq(orders.offerId, offerId),
          eq(orders.status, "PENDING"),
        ),
      );
    return order || null;
  }
}
