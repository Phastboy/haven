import { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import { UnauthorizedError } from "../../auth/domain/errors";

export class MarkThreadAsReadUseCase {
  constructor(private readonly repository: SqlMessageRepository) {}

  async execute(threadId: string, userId: string): Promise<void> {
    const thread = await this.repository.findThreadById(threadId);
    if (!thread) {
      throw new Error("Thread not found");
    }

    if (thread.participant1Id !== userId && thread.participant2Id !== userId) {
      throw new UnauthorizedError("User is not a participant in this thread");
    }

    await this.repository.markMessagesAsRead(threadId, userId);
  }
}
