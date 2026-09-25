import type { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import type { ThreadRecord } from "../../database/schema";
import { SelfThreadError } from "../domain/errors";

export class CreateThreadUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(user1Id: string, user2Id: string): Promise<ThreadRecord> {
    if (user1Id === user2Id) {
      throw new SelfThreadError();
    }

    const p1 = user1Id < user2Id ? user1Id : user2Id;
    const p2 = user1Id < user2Id ? user2Id : user1Id;

    const existingThread = await this.messageRepo.findThreadByParticipants(p1, p2);
    if (existingThread) {
      return existingThread;
    }

    try {
      return await this.messageRepo.createThread(p1, p2);
    } catch (e: unknown) {
      if (
        typeof e === "object" &&
        e !== null &&
        "code" in e &&
        (e as { code: string }).code === "23505"
      ) {
        const conflict = await this.messageRepo.findThreadByParticipants(p1, p2);
        if (conflict) return conflict;
      }
      throw e;
    }
  }
}
