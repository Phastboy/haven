import type { DB } from "../../../database/db";
import { accounts } from "../../../database/schema";
import { eq } from "drizzle-orm";
import { toIso } from "../../../database/map";
import type { IAccountRepository } from "../../domain/ports/IAccountRepository";
import type { Account, CreateAccountDTO } from "../../domain/account.schema";

export class AccountRepository implements IAccountRepository {
  readonly #db: DB;

  constructor(db: DB) {
    this.#db = db;
  }

  async create(data: CreateAccountDTO): Promise<Account> {
    const id = crypto.randomUUID();
    const rows = await this.#db
      .insert(accounts)
      .values({
        id,
        email: data.email,
        emailVerified: data.emailVerified,
      })
      .returning();
    return this.#toAccount(rows[0]!);
  }

  async findById(id: string): Promise<Account | null> {
    const row = await this.#db.query.accounts.findFirst({
      where: eq(accounts.id, id),
    });
    return row ? this.#toAccount(row) : null;
  }

  async findByEmail(email: string): Promise<Account | null> {
    const row = await this.#db.query.accounts.findFirst({
      where: eq(accounts.email, email),
    });
    return row ? this.#toAccount(row) : null;
  }

  async markEmailVerified(id: string): Promise<void> {
    await this.#db
      .update(accounts)
      .set({ emailVerified: true, updatedAt: new Date() })
      .where(eq(accounts.id, id));
  }

  #toAccount(row: typeof accounts.$inferSelect): Account {
    return {
      id: row.id,
      email: row.email,
      emailVerified: row.emailVerified,
      createdAt: toIso(row.createdAt),
      updatedAt: toIso(row.updatedAt),
    };
  }
}
