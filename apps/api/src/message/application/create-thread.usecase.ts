import type { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import type { ThreadRecord } from "../../database/schema";
import { SelfThreadError } from "../domain/errors";

export class CreateThreadUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(user1Id: string, user2Id: string): Promise<ThreadRecord> {
    if (user1Id === user2Id) {
      throw new SelfThreadError();
    }

    const existingThread = await this.messageRepo.findThreadByParticipants(user1Id, user2Id);
    if (existingThread) {
      return existingThread;
    }

    return this.messageRepo.createThread(user1Id, user2Id);
  }
}
