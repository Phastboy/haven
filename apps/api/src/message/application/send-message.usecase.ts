import type { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import type { MessageContextRecord } from "../../database/schema";
import type { MessageRecord } from "../../database/schema";
import { ThreadNotFoundError, UnauthorizedThreadAccessError, TooManyContextsError } from "../domain/errors";

import type { IEventBus } from "../../shared/domain/event-bus.interface";

export class SendMessageUseCase {
  constructor(
    private readonly messageRepo: SqlMessageRepository,
    private readonly eventBus: IEventBus,
  ) {}

  async execute(threadId: string, senderId: string, content: string, contextOfferIds?: string[]): Promise<MessageRecord & { contexts: MessageContextRecord[] }> {
    if (contextOfferIds && contextOfferIds.length > 5) {
      throw new TooManyContextsError();
    }

    const thread = await this.messageRepo.findThreadById(threadId);
    if (!thread) {
      throw new ThreadNotFoundError();
    }

    if (thread.participant1Id !== senderId && thread.participant2Id !== senderId) {
      throw new UnauthorizedThreadAccessError();
    }

    const message = await this.messageRepo.sendMessage(threadId, senderId, content, contextOfferIds);

    // Determine receiver
    const receiverId =
      thread.participant1Id === senderId ? thread.participant2Id : thread.participant1Id;

    // Emit event for real-time delivery
    try {
      await this.eventBus.publish("message.created", {
        threadId,
        senderId,
        receiverId,
        message,
      });
    } catch (error) {
      console.error("[SendMessageUseCase] Failed to publish message.created event:", error);
    }

    return message;
  }
}
