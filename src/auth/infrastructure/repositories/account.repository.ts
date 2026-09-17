import { db } from '../../../database/db';
import { IAccountRepository } from '../../domain/ports/IAccountRepository';
import { Account, CreateAccountDTO } from '../../domain/account.schema';

export class AccountRepository implements IAccountRepository {
  async create(data: CreateAccountDTO): Promise<Account> {
    const id = crypto.randomUUID();
    const rows = await db`
      INSERT INTO "Account" (id, email, "emailVerified")
      VALUES (${id}, ${data.email}, ${data.emailVerified})
      RETURNING *
    `;
    return this.toEntity(rows[0]);
  }

  async findById(id: string): Promise<Account | null> {
    const rows = await db`SELECT * FROM "Account" WHERE id = ${id}`;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  async findByEmail(email: string): Promise<Account | null> {
    const rows = await db`SELECT * FROM "Account" WHERE email = ${email}`;
    return rows.length > 0 ? this.toEntity(rows[0]) : null;
  }

  async markEmailVerified(id: string): Promise<void> {
    await db`
      UPDATE "Account"
      SET "emailVerified" = true, "updatedAt" = CURRENT_TIMESTAMP
      WHERE id = ${id}
    `;
  }

  private toEntity(row: any): Account {
    return {
      id: row.id,
      email: row.email,
      emailVerified: row.emailVerified,
      createdAt: row.createdAt instanceof Date ? row.createdAt.toISOString() : row.createdAt,
      updatedAt: row.updatedAt instanceof Date ? row.updatedAt.toISOString() : row.updatedAt,
    };
  }
}
