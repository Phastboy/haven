import type { IOfferRepository } from "../domain/offer.repository";
import type { Offer } from "../domain/offer.schema";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class ListUserOffersUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  /**
   * @param userId      The profile whose offers to list.
   * @param requesterId The account making the request (undefined = unauthenticated).
   *                    If requesterId === userId the caller is the owner and sees all statuses.
   * @param limit       Max rows to return (capped at 100 in the repository).
   * @param offset      Pagination offset.
   */
  async execute(
    userId: string,
    requesterId?: string,
    limit = 50,
    offset = 0,
  ): Promise<PaginatedResponse<Offer>> {
    const data =
      requesterId === userId
        ? await this.repository.findByUserId(userId, limit, offset)
        : await this.repository.findActiveByUserId(userId, limit, offset);

    return createPaginatedResponse(data, {
      hasMore: data.length === limit,
    });
  }
}
