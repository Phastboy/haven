import type { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import type { MessageRecord } from "../../database/schema";
import { ThreadNotFoundError, UnauthorizedThreadAccessError } from "../domain/errors";

export class GetMessagesUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(threadId: string, userId: string): Promise<MessageRecord[]> {
    const thread = await this.messageRepo.findThreadById(threadId);
    if (!thread) {
      throw new ThreadNotFoundError();
    }

    if (thread.participant1Id !== userId && thread.participant2Id !== userId) {
      throw new UnauthorizedThreadAccessError();
    }

    return this.messageRepo.getThreadMessages(threadId);
  }
}
