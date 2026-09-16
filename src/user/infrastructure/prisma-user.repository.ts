import { randomUUIDv7 } from 'bun';
import { db } from '../../prisma/db';
import type { IUserRepository } from '../domain/user.repository';
import type { User, CreateUserData, UpdateUserData } from '../domain/user.entity';

/**
 * Prisma 8 implementation of IUserRepository.
 * This is the only file in the user feature that imports from the ORM.
 */
export class PrismaUserRepository implements IUserRepository {
  async findById(id: string): Promise<User | null> {
    return db.orm.public.User.first({ id });
  }

  async findByEmail(email: string): Promise<User | null> {
    return db.orm.public.User
      .where((u) => u.email.eq(email))
      .first();
  }

  async findAll(): Promise<User[]> {
    return db.orm.public.User
      .orderBy((u) => u.createdAt.desc())
      .all();
  }

  async create(data: CreateUserData): Promise<User> {
    return db.orm.public.User.create({
      id: randomUUIDv7(),
      email: data.email,
      username: data.username ?? null,
      name: data.name ?? null,
    });
  }

  async update(id: string, data: UpdateUserData): Promise<User> {
    const rows = await db.orm.public.User
      .where({ id })
      .select('id', 'email', 'username', 'name', 'createdAt', 'updatedAt')
      .update(data);
    // update() returns the updated rows; we updated exactly one.
    return rows[0]!;
  }

  async delete(id: string): Promise<void> {
    await db.orm.public.User.where({ id }).delete();
  }
}
