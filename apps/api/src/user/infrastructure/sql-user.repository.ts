import type { DB } from "../../database/db";
import { users } from "../../database/schema";
import { eq, desc } from "drizzle-orm";
import { toIso } from "../../database/map";
import type { IUserRepository } from "../domain/user.repository";
import type { User, CreateUserData, UpdateUserData } from "../domain/user.entity";
import { NotFoundError } from "../../shared/errors";

export class SqlUserRepository implements IUserRepository {
  readonly #db: DB;

  constructor(db: DB) {
    this.#db = db;
  }

  async findById(id: string): Promise<User | null> {
    const row = await this.#db.query.users.findFirst({
      where: eq(users.id, id),
    });
    return row ? this.#toUser(row) : null;
  }

  async findByAccountId(accountId: string): Promise<User | null> {
    const row = await this.#db.query.users.findFirst({
      where: eq(users.accountId, accountId),
    });
    return row ? this.#toUser(row) : null;
  }

  async findAll(): Promise<User[]> {
    const rows = await this.#db.query.users.findMany({
      orderBy: [desc(users.createdAt)],
    });
    return rows.map((r) => this.#toUser(r));
  }

  async create(data: CreateUserData): Promise<User> {
    const id = crypto.randomUUID();
    const rows = await this.#db
      .insert(users)
      .values({
        id,
        accountId: data.accountId,
        username: data.username ?? null,
        name: data.name ?? null,
        bio: data.bio ?? null,
        profilePictureUrl: data.profilePictureUrl ?? null,
      })
      .returning();
    return this.#toUser(rows[0]!);
  }

  async update(id: string, data: UpdateUserData): Promise<User> {
    const updates: Partial<typeof users.$inferInsert> = {
      updatedAt: new Date(),
    };

    if (data.username !== undefined) updates.username = data.username;
    if (data.name !== undefined) updates.name = data.name;
    if (data.bio !== undefined) updates.bio = data.bio;
    if (data.profilePictureUrl !== undefined) updates.profilePictureUrl = data.profilePictureUrl;

    // If only updatedAt is in the object, technically we can just update that or skip.
    // The previous implementation required at least one field to be updated or it checked existence.
    if (Object.keys(updates).length === 1) {
      // Only updatedAt
      const existing = await this.findById(id);
      if (!existing) throw new Error(`User ${id} disappeared during update`);
      return existing;
    }

    const rows = await this.#db.update(users).set(updates).where(eq(users.id, id)).returning();
    const row = rows[0];

    if (!row) throw new NotFoundError("User", id);
    return this.#toUser(row);
  }

  async delete(id: string): Promise<void> {
    await this.#db.delete(users).where(eq(users.id, id));
  }

  #toUser(row: typeof users.$inferSelect): User {
    return {
      id: row.id,
      accountId: row.accountId,
      username: row.username ?? null,
      name: row.name ?? null,
      bio: row.bio ?? null,
      profilePictureUrl: row.profilePictureUrl ?? null,
      createdAt: toIso(row.createdAt),
      updatedAt: toIso(row.updatedAt),
    };
  }
}
