import { config } from "../../../config";
import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { createDb } from "../../../database/db";
const db = createDb(config);

import { users, accounts } from "../../../database/schema";
import { SqlMessageRepository } from "../sql-message.repository";
import { randomUUID } from "crypto";
import { eq } from "drizzle-orm";

describe("SqlMessageRepository Integration", () => {
  const repo = new SqlMessageRepository(db);
  let user1Id: string;
  let user2Id: string;
  let account1Id: string;
  let account2Id: string;

  beforeAll(async () => {
    account1Id = randomUUID();
    await db
      .insert(accounts)
      .values({ id: account1Id, email: `test1_${randomUUID()}@example.com` });
    user1Id = randomUUID();
    await db
      .insert(users)
      .values({ id: user1Id, accountId: account1Id, username: `user1_${randomUUID()}` });

    account2Id = randomUUID();
    await db
      .insert(accounts)
      .values({ id: account2Id, email: `test2_${randomUUID()}@example.com` });
    user2Id = randomUUID();
    await db
      .insert(users)
      .values({ id: user2Id, accountId: account2Id, username: `user2_${randomUUID()}` });
  });

  afterAll(async () => {
    await db.delete(accounts).where(eq(accounts.id, account1Id));
    await db.delete(accounts).where(eq(accounts.id, account2Id));
  });

  it("should create and find a thread", async () => {
    const thread = await repo.createThread(user1Id, user2Id);
    expect(thread.id).toBeDefined();

    const foundById = await repo.findThreadById(thread.id);
    expect(foundById?.id).toBe(thread.id);

    const foundByParticipants = await repo.findThreadByParticipants(user1Id, user2Id);
    expect(foundByParticipants?.id).toBe(thread.id);
  });

  it("should send and retrieve messages", async () => {
    const thread = await repo.createThread(user1Id, user2Id);

    await repo.sendMessage(thread.id, user1Id, "Hello");
    await repo.sendMessage(thread.id, user2Id, "World");

    const messages = await repo.getThreadMessages(thread.id);
    expect(messages).toHaveLength(2);
    expect(messages[0]?.content).toBe("Hello");
    expect(messages[1]?.content).toBe("World");
  });

  it("should retrieve user threads with latest message", async () => {
    const thread = await repo.createThread(user1Id, user2Id);
    await repo.sendMessage(thread.id, user1Id, "Latest test");

    const threads = await repo.getUserThreads(user1Id);
    const found = threads.find((t) => t.id === thread.id);
    expect(found).toBeDefined();
    expect(found?.latestMessage?.content).toBe("Latest test");
    expect(found?.participant2?.id).toBe(user2Id);
  });
});
