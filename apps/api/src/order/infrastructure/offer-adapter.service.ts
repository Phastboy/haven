import { db } from "../../database/db";
import { offers } from "../../database/schema";
import { eq } from "drizzle-orm";
import type { IOfferService } from "../application/create-order.usecase";
import type { IOfferOwnerService } from "../application/update-order-status.usecase";

export class OfferAdapterService implements IOfferService, IOfferOwnerService {
  async getOfferPriceAndOwnerAndStatus(
    offerId: string,
  ): Promise<{ price: number; ownerId: string; status: string } | null> {
    const [offer] = await db
      .select({
        price: offers.price,
        ownerId: offers.userId,
        status: offers.status,
      })
      .from(offers)
      .where(eq(offers.id, offerId));
    return offer || null;
  }

  async getOfferOwnerId(offerId: string): Promise<string | null> {
    const [offer] = await db
      .select({ ownerId: offers.userId })
      .from(offers)
      .where(eq(offers.id, offerId));
    return offer ? offer.ownerId : null;
  }
}
