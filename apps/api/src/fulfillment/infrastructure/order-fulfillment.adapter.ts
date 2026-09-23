import { eq } from "drizzle-orm";
import { db } from "../../database/db";
import { orders, offers } from "../../database/schema";
import { IOrderFulfillmentService } from "../application/order-fulfillment.service.interface";

export class OrderFulfillmentAdapter implements IOrderFulfillmentService {
  async getOrderDetails(
    orderId: string,
  ): Promise<{ id: string; requesterId: string; offerId: string; status: string } | null> {
    const [order] = await db
      .select({
        id: orders.id,
        requesterId: orders.requesterId,
        offerId: orders.offerId,
        status: orders.status,
      })
      .from(orders)
      .where(eq(orders.id, orderId));

    return order || null;
  }

  async getOfferTypeAndOwner(
    offerId: string,
  ): Promise<{ offerType: string; ownerId: string } | null> {
    const [offer] = await db
      .select({
        offerType: offers.offerType,
        ownerId: offers.userId,
      })
      .from(offers)
      .where(eq(offers.id, offerId));

    return offer || null;
  }

  async updateOrderStatus(orderId: string, status: string): Promise<void> {
    await db
      .update(orders)
      .set({
        status: status as "PENDING" | "ACCEPTED" | "REJECTED" | "COMPLETED" | "CANCELLED",
        updatedAt: new Date(),
      })
      .where(eq(orders.id, orderId));
  }
}
