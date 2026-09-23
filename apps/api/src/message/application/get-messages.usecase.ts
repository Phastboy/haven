import type { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import type { MessageRecord } from "../../database/schema";
import { ThreadNotFoundError, UnauthorizedThreadAccessError } from "../domain/errors";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class GetMessagesUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(threadId: string, userId: string): Promise<PaginatedResponse<MessageRecord>> {
    const thread = await this.messageRepo.findThreadById(threadId);
    if (!thread) {
      throw new ThreadNotFoundError();
    }

    if (thread.participant1Id !== userId && thread.participant2Id !== userId) {
      throw new UnauthorizedThreadAccessError();
    }

    const data = await this.messageRepo.getThreadMessages(threadId);
    return createPaginatedResponse(data, { total: data.length });
  }
}
