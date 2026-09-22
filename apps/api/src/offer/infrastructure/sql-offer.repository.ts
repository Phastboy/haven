import { IOfferRepository } from "../domain/offer.repository";
import {
  Offer,
  CreateOfferData,
  UpdateOfferData,
  OfferStatus,
  OfferType,
} from "../domain/offer.schema";
import { db } from "../../database/db";
import { offers } from "../../database/schema";
import { randomUUID } from "crypto";
import { and, eq, ne } from "drizzle-orm";

export class SqlOfferRepository implements IOfferRepository {
  async create(userId: string, data: CreateOfferData): Promise<Offer> {
    const id = randomUUID();
    const [record] = await db
      .insert(offers)
      .values({
        id,
        userId,
        title: data.title,
        description: data.description ?? null,
        price: data.price ?? 0,
        offerType: data.offerType ?? "PRODUCT",
        images: data.images ?? null,
        status: "ACTIVE",
      })
      .returning();

    return this.mapToDomain(record!);
  }

  async findById(id: string): Promise<Offer | null> {
    const [record] = await db.select().from(offers).where(eq(offers.id, id));
    if (!record) return null;
    return this.mapToDomain(record);
  }

  async findByUserId(userId: string): Promise<Offer[]> {
    const records = await db.select().from(offers).where(eq(offers.userId, userId));
    return records.map(this.mapToDomain);
  }

  async findActiveByUserId(userId: string): Promise<Offer[]> {
    const records = await db
      .select()
      .from(offers)
      .where(and(eq(offers.userId, userId), ne(offers.status, "ARCHIVED")));
    return records.map(this.mapToDomain);
  }

  async update(id: string, data: UpdateOfferData): Promise<Offer | null> {
    const updateData: Partial<typeof offers.$inferInsert> = {
      updatedAt: new Date(),
    };

    if (data.title !== undefined) updateData.title = data.title;
    if (data.description !== undefined) updateData.description = data.description;
    if (data.price !== undefined) updateData.price = data.price;
    if (data.offerType !== undefined) updateData.offerType = data.offerType;
    if (data.images !== undefined) updateData.images = data.images;

    const [record] = await db.update(offers).set(updateData).where(eq(offers.id, id)).returning();

    if (!record) return null;
    return this.mapToDomain(record);
  }

  async delete(id: string): Promise<boolean> {
    const [record] = await db
      .update(offers)
      .set({ status: "ARCHIVED", updatedAt: new Date() })
      .where(eq(offers.id, id))
      .returning();

    return !!record;
  }

  private mapToDomain(record: typeof offers.$inferSelect): Offer {
    return {
      id: record.id,
      userId: record.userId,
      title: record.title,
      description: record.description,
      price: record.price,
      status: record.status as OfferStatus,
      offerType: record.offerType as OfferType,
      images: record.images as string[] | null,
      createdAt: record.createdAt,
      updatedAt: record.updatedAt,
    };
  }
}
