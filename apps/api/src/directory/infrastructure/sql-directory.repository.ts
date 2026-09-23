import { IDirectoryRepository } from "../domain/directory.repository";
import { DirectoryOffer, DirectoryUser } from "../domain/directory.schema";
import { db } from "../../database/db";
import { offers, users } from "../../database/schema";
import { eq, desc, inArray } from "drizzle-orm";

export class SqlDirectoryRepository implements IDirectoryRepository {
  async getActiveOffers(limit: number = 50, offset: number = 0): Promise<DirectoryOffer[]> {
    const records = await db
      .select()
      .from(offers)
      .where(eq(offers.status, "ACTIVE"))
      .orderBy(desc(offers.createdAt))
      .limit(limit)
      .offset(offset);

    return records.map((record) => ({
      id: record.id,
      userId: record.userId,
      title: record.title,
      description: record.description,
      price: record.price,
      offerType: record.offerType,
      images: record.images as string[] | null,
      createdAt: record.createdAt,
    }));
  }

  async getUsersByIds(userIds: string[]): Promise<DirectoryUser[]> {
    if (userIds.length === 0) return [];

    const records = await db.select().from(users).where(inArray(users.id, userIds));

    return records.map((record) => ({
      id: record.id,
      username: record.username,
      name: record.name,
      profilePictureUrl: record.profilePictureUrl,
    }));
  }
}
