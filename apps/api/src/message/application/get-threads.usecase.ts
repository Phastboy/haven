import {
  SqlMessageRepository,
  ThreadWithParticipants,
} from "../infrastructure/sql-message.repository";

export class GetThreadsUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(userId: string): Promise<ThreadWithParticipants[]> {
    return this.messageRepo.getUserThreads(userId);
  }
}
