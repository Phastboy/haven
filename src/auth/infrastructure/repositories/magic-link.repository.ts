import { db } from '../../../database/db';
import { CreateMagicLinkDTO, IMagicLinkRepository } from '../../domain/ports/IMagicLinkRepository';
import { MagicLink } from '../../domain/magic-link.schema';

export class MagicLinkRepository implements IMagicLinkRepository {
  async create(data: CreateMagicLinkDTO): Promise<MagicLink> {
    const id = crypto.randomUUID();
    const rows = await db`
      INSERT INTO "MagicLink" (id, email, token, "expiresAt", "usedAt")
      VALUES (${id}, ${data.email}, ${data.token}, ${data.expiresAt}, null)
      RETURNING *
    `;
    return this.toEntity(rows[0]);
  }

  async findByToken(token: string): Promise<MagicLink | null> {
    const rows = await db`SELECT * FROM "MagicLink" WHERE token = ${token}`;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  async markUsed(id: string, usedAt: string): Promise<boolean> {
    const rows = await db`
      UPDATE "MagicLink"
      SET "usedAt" = ${usedAt}
      WHERE id = ${id} AND "usedAt" IS NULL
      RETURNING id
    `;
    return rows.length > 0;
  }

  private toEntity(row: any): MagicLink {
    const res: any = {
      id: row.id,
      email: row.email,
      token: row.token,
      expiresAt: row.expiresAt instanceof Date ? row.expiresAt.toISOString() : row.expiresAt,
      createdAt: row.createdAt instanceof Date ? row.createdAt.toISOString() : row.createdAt,
    };
    if (row.usedAt) {
      res.usedAt = row.usedAt instanceof Date ? row.usedAt.toISOString() : row.usedAt;
    }
    return res;
  }
}
