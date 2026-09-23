import { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import { MessageRecord } from "../../database/schema";
import { ThreadNotFoundError, UnauthorizedThreadAccessError } from "../domain/errors";

import { IEventBus } from "../../shared/domain/event-bus.interface";

export class SendMessageUseCase {
  constructor(
    private readonly messageRepo: SqlMessageRepository,
    private readonly eventBus: IEventBus,
  ) {}

  async execute(threadId: string, senderId: string, content: string): Promise<MessageRecord> {
    const thread = await this.messageRepo.findThreadById(threadId);
    if (!thread) {
      throw new ThreadNotFoundError();
    }

    if (thread.participant1Id !== senderId && thread.participant2Id !== senderId) {
      throw new UnauthorizedThreadAccessError();
    }

    const message = await this.messageRepo.sendMessage(threadId, senderId, content);

    // Determine receiver
    const receiverId =
      thread.participant1Id === senderId ? thread.participant2Id : thread.participant1Id;

    // Emit event for real-time delivery
    this.eventBus.publish("message.created", {
      threadId,
      senderId,
      receiverId,
      message,
    });

    return message;
  }
}
