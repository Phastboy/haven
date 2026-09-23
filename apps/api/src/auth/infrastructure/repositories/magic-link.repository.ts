import { db } from "../../../database/db";
import { magicLinks } from "../../../database/schema";
import { eq, and, isNull } from "drizzle-orm";
import { toIso } from "../../../database/map";
import type { CreateMagicLinkDTO, IMagicLinkRepository } from "../../domain/ports/IMagicLinkRepository";
import type { MagicLink } from "../../domain/magic-link.schema";

export class MagicLinkRepository implements IMagicLinkRepository {
  async create(data: CreateMagicLinkDTO): Promise<MagicLink> {
    const id = crypto.randomUUID();
    const rows = await db
      .insert(magicLinks)
      .values({
        id,
        email: data.email,
        token: data.token,
        expiresAt: new Date(data.expiresAt),
        usedAt: null,
      })
      .returning();
    return this.#toMagicLink(rows[0]!);
  }

  async findByToken(token: string): Promise<MagicLink | null> {
    const row = await db.query.magicLinks.findFirst({
      where: eq(magicLinks.token, token),
    });
    return row ? this.#toMagicLink(row) : null;
  }

  async markUsed(id: string, usedAt: string): Promise<boolean> {
    const rows = await db
      .update(magicLinks)
      .set({ usedAt: new Date(usedAt) })
      .where(and(eq(magicLinks.id, id), isNull(magicLinks.usedAt)))
      .returning({ id: magicLinks.id });
    return rows.length > 0;
  }

  #toMagicLink(row: typeof magicLinks.$inferSelect): MagicLink {
    return {
      id: row.id,
      email: row.email,
      token: row.token,
      expiresAt: toIso(row.expiresAt),
      createdAt: toIso(row.createdAt),
      ...(row.usedAt ? { usedAt: toIso(row.usedAt) } : {}),
    };
  }
}
