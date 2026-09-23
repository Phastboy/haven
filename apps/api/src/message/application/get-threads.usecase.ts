import type {
  SqlMessageRepository,
  ThreadWithParticipants,
} from "../infrastructure/sql-message.repository";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class GetThreadsUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(userId: string): Promise<PaginatedResponse<ThreadWithParticipants>> {
    const data = await this.messageRepo.getUserThreads(userId);
    return createPaginatedResponse(data, { total: data.length });
  }
}
