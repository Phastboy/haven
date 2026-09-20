import { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import { MessageRecord } from "../../database/schema";
import { ThreadNotFoundError, UnauthorizedThreadAccessError } from "../domain/errors";

export class SendMessageUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(threadId: string, senderId: string, content: string): Promise<MessageRecord> {
    const thread = await this.messageRepo.findThreadById(threadId);
    if (!thread) {
      throw new ThreadNotFoundError();
    }

    if (thread.participant1Id !== senderId && thread.participant2Id !== senderId) {
      throw new UnauthorizedThreadAccessError();
    }

    return this.messageRepo.sendMessage(threadId, senderId, content);
  }
}
