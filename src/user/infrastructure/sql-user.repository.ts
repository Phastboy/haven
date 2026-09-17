import { db } from '../../database/db';
import type { IUserRepository } from '../domain/user.repository';
import type { User, CreateUserData, UpdateUserData } from '../domain/user.entity';
import { NotFoundError } from '../../shared/errors';

export class SqlUserRepository implements IUserRepository {
  async findById(id: string): Promise<User | null> {
    const rows = await db`SELECT * FROM "User" WHERE id = ${id}`;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  async findByEmail(email: string): Promise<User | null> {
    const rows = await db`SELECT * FROM "User" WHERE email = ${email}`;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  async findAll(): Promise<User[]> {
    const rows = await db`SELECT * FROM "User" ORDER BY "createdAt" DESC`;
    return rows.map((r: any) => this.toEntity(r));
  }

  async create(data: CreateUserData): Promise<User> {
    const id = crypto.randomUUID();
    const rows = await db`
      INSERT INTO "User" (id, email, username, name)
      VALUES (${id}, ${data.email}, ${data.username ?? null}, ${data.name ?? null})
      RETURNING *
    `;
    return this.toEntity(rows[0]);
  }

  async update(id: string, data: UpdateUserData): Promise<User> {
    const updates = [];
    // email update not supported
    if (data.username !== undefined) updates.push(db`username = ${data.username}`);
    if (data.name !== undefined) updates.push(db`name = ${data.name}`);
    
    if (updates.length === 0) {
      const existing = await this.findById(id);
      if (!existing) throw new Error(`User ${id} disappeared during update`);
      return existing;
    }
    
    // Bun's SQL tag can combine query pieces? We can manually do it or just execute standard update.
    // Wait, Bun's SQL driver doesn't support array joining easily like that yet. Let's do it safely:
    // We can use a transaction or build the query object. 
    // For simplicity, let's just do dynamic query building. Actually, standard Postgres allows COALESCE or just updating the exact fields.
    // Let's do a simple approach.
    const row = await db.begin(async (tx) => {
      let query = 'UPDATE "User" SET ';
      const values: any[] = [];
      const parts = [];
      let i = 1;
      // email update not supported
      if (data.username !== undefined) { parts.push(`username = $${i++}`); values.push(data.username); }
      if (data.name !== undefined) { parts.push(`name = $${i++}`); values.push(data.name); }
      
      parts.push(`"updatedAt" = CURRENT_TIMESTAMP`);
      query += parts.join(', ') + ` WHERE id = $${i} RETURNING *`;
      values.push(id);
      
      const res = await tx.unsafe(query, values);
      return res[0];
    });

    if (!row) throw new NotFoundError('User', id);
    return this.toEntity(row);
  }

  async delete(id: string): Promise<void> {
    await db`DELETE FROM "User" WHERE id = ${id}`;
  }

  private toEntity(row: any): User {
    return {
      id: row.id,
      email: row.email,
      username: row.username,
      name: row.name,
      createdAt: row.createdAt instanceof Date ? row.createdAt.toISOString() : row.createdAt,
      updatedAt: row.updatedAt instanceof Date ? row.updatedAt.toISOString() : row.updatedAt,
    };
  }
}
