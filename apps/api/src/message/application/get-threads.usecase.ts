import type {
  SqlMessageRepository,
  ThreadWithParticipants,
} from "../infrastructure/sql-message.repository";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class GetThreadsUseCase {
  constructor(private readonly messageRepo: SqlMessageRepository) {}

  async execute(userId: string, limit = 50, offset = 0): Promise<PaginatedResponse<ThreadWithParticipants>> {
    const data = await this.messageRepo.getUserThreads(userId, limit, offset);
    return createPaginatedResponse(data, { hasMore: data.length === limit });
  }
}
