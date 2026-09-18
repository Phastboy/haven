// oxlint-disable typescript/no-explicit-any
import { sql } from 'drizzle-orm';
import { expect, test, describe } from 'bun:test';
import { SqlUserRepository } from '../sql-user.repository';
import { AccountRepository } from '../../../auth/infrastructure/repositories/account.repository';
import { db } from '../../../database/db';
import { NotFoundError } from '../../../shared/errors';

describe('SqlUserRepository Integration', () => {
  const repo = new SqlUserRepository();
  const accountRepo = new AccountRepository();

  test('should create a new user profile', async () => {
    const testEmail = `test-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({
      email: testEmail,
      emailVerified: false,
    });

    const user = await repo.create({
      accountId: account.id,
      username: 'testuser',
      name: 'Test User',
      bio: 'Hello world',
      profilePictureUrl: 'http://example.com/pic.png',
    });

    expect(user.id).toBeDefined();
    expect(user.accountId).toBe(account.id);
    expect(user.username).toBe('testuser');
    expect(user.name).toBe('Test User');
    expect(user.bio).toBe('Hello world');
    expect(user.profilePictureUrl).toBe('http://example.com/pic.png');

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`); // Cascades to User
  });

  test('should find user by accountId', async () => {
    const testEmail = `test-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({ email: testEmail, emailVerified: false });
    const user = await repo.create({ accountId: account.id });

    const found = await repo.findByAccountId(account.id);
    expect(found).not.toBeNull();
    expect(found!.id).toBe(user.id);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test('should return null for non-existent accountId', async () => {
    const user = await repo.findByAccountId('does-not-exist');
    expect(user).toBeNull();
  });

  test('should find user by id', async () => {
    const testEmail = `test-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({ email: testEmail, emailVerified: false });
    const user = await repo.create({ accountId: account.id });

    const found = await repo.findById(user.id);
    expect(found).not.toBeNull();
    expect(found!.accountId).toBe(account.id);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test('should list all users', async () => {
    const testEmail = `test-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({ email: testEmail, emailVerified: false });
    await repo.create({ accountId: account.id });

    const users = await repo.findAll();
    expect(users.length).toBeGreaterThanOrEqual(1);

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test('should update a user', async () => {
    const testEmail = `test-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({ email: testEmail, emailVerified: false });
    const user = await repo.create({ accountId: account.id, username: 'olduser' });

    const updated = await repo.update(user.id, { username: 'newuser', name: 'New Name' });
    expect(updated.username).toBe('newuser');
    expect(updated.name).toBe('New Name');

    const fetched = await repo.findById(user.id);
    expect(fetched!.username).toBe('newuser');

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });

  test('should delete a user directly', async () => {
    const testEmail = `test-user-${crypto.randomUUID()}@example.com`;
    const account = await accountRepo.create({ email: testEmail, emailVerified: false });
    const user = await repo.create({ accountId: account.id });

    await repo.delete(user.id);

    const found = await repo.findById(user.id);
    expect(found).toBeNull();

    await db.execute(sql`DELETE FROM "Account" WHERE "id" = ${account.id}`);
  });
});
