import type { DB } from "../../database/db";
import type { ThreadRecord, MessageContextRecord } from "../../database/schema";
import type { MessageRecord } from "../../database/schema";
import { threads, messages, messageContexts } from "../../database/schema";
import { eq, or, and, desc, asc } from "drizzle-orm";
import { randomUUID } from "crypto";

/** Narrows an unknown catch value to a DB error object with a code field. */
function isDbError(e: unknown): e is { code: string } {
  return (
    typeof e === "object" &&
    e !== null &&
    "code" in e &&
    typeof (e as Record<string, unknown>)["code"] === "string"
  );
}

export interface ThreadWithParticipants extends ThreadRecord {
  participant1: { id: string; name: string | null; profilePictureUrl: string | null };
  participant2: { id: string; name: string | null; profilePictureUrl: string | null };
  latestMessage?: MessageRecord | undefined;
}

export class SqlMessageRepository {
  readonly #db: DB;

  constructor(db: DB) {
    this.#db = db;
  }

  async findThreadById(threadId: string): Promise<ThreadRecord | null> {
    const [thread] = await this.#db.select().from(threads).where(eq(threads.id, threadId));
    return thread || null;
  }

  async findThreadByParticipants(user1Id: string, user2Id: string): Promise<ThreadRecord | null> {
    const [thread] = await this.#db
      .select()
      .from(threads)
      .where(
        or(
          and(eq(threads.participant1Id, user1Id), eq(threads.participant2Id, user2Id)),
          and(eq(threads.participant1Id, user2Id), eq(threads.participant2Id, user1Id)),
        ),
      );
    return thread || null;
  }

  async createThread(participant1Id: string, participant2Id: string): Promise<ThreadRecord> {
    try {
      const [thread] = await this.#db
        .insert(threads)
        .values({
          id: randomUUID(),
          participant1Id,
          participant2Id,
        })
        .returning();
      return thread!;
    } catch (e: unknown) {
      if (isDbError(e) && e.code === "23503") {
        const { ParticipantNotFoundError } = await import("../domain/errors");
        throw new ParticipantNotFoundError();
      }
      throw e;
    }
  }

  async getUserThreads(userId: string, limit: number, offset: number): Promise<ThreadWithParticipants[]> {
    const userThreads = await this.#db.query.threads.findMany({
      where: or(eq(threads.participant1Id, userId), eq(threads.participant2Id, userId)),
      limit,
      offset,
      with: {
        participant1: {
          columns: { id: true, name: true, profilePictureUrl: true },
        },
        participant2: {
          columns: { id: true, name: true, profilePictureUrl: true },
        },
        messages: {
          orderBy: [desc(messages.createdAt)],
          limit: 1,
        },
      },
      orderBy: [desc(threads.updatedAt)],
    });

    return userThreads.map((t) => ({
      ...t,
      latestMessage: t.messages[0],
    }));
  }

  async getThreadMessages(
    threadId: string,
    limit: number,
    offset: number,
  ): Promise<(MessageRecord & { contexts: MessageContextRecord[] })[]> {
    return this.#db.query.messages.findMany({
      where: eq(messages.threadId, threadId),
      limit,
      offset,
      with: {
        contexts: true,
      },
      orderBy: [asc(messages.createdAt)],
    });
  }

  async sendMessage(
    threadId: string,
    senderId: string,
    content: string,
    contextOfferIds?: string[],
  ): Promise<MessageRecord & { contexts: MessageContextRecord[] }> {
    try {
      return await this.#db.transaction(async (tx) => {
        const [message] = await tx
          .insert(messages)
          .values({
            id: randomUUID(),
            threadId,
            senderId,
            content,
          })
          .returning();

        let contexts: MessageContextRecord[] = [];
        if (contextOfferIds && contextOfferIds.length > 0) {
          contexts = await tx
            .insert(messageContexts)
            .values(
              contextOfferIds.map((offerId) => ({
                id: randomUUID(),
                messageId: message!.id,
                offerId,
              })),
            )
            .returning();
        }

        // Update thread's updatedAt
        await tx.update(threads).set({ updatedAt: new Date() }).where(eq(threads.id, threadId));

        return { ...message!, contexts };
      });
    } catch (e: unknown) {
      if (isDbError(e) && e.code === "23503" && contextOfferIds?.length) {
        const { OfferNotFoundError } = await import("../../offer/domain/errors");
        throw new OfferNotFoundError();
      }
      throw e;
    }
  }

  async markMessagesAsRead(threadId: string, userId: string): Promise<void> {
    const { ne, isNull } = await import("drizzle-orm");
    await this.#db
      .update(messages)
      .set({ readAt: new Date() })
      .where(
        and(
          eq(messages.threadId, threadId),
          ne(messages.senderId, userId),
          isNull(messages.readAt),
        ),
      );
  }
}
