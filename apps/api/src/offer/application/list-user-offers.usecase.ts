import type { IOfferRepository } from "../domain/offer.repository";
import type { Offer } from "../domain/offer.schema";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class ListUserOffersUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  /**
   * @param userId     The profile whose offers to list.
   * @param requesterId  The account making the request (undefined = unauthenticated).
   *                   If requesterId === userId the caller is the owner and sees all statuses.
   */
  async execute(userId: string, requesterId?: string): Promise<PaginatedResponse<Offer>> {
    const data =
      requesterId === userId
        ? await this.repository.findByUserId(userId)
        : await this.repository.findActiveByUserId(userId);

    return createPaginatedResponse(data, { total: data.length });
  }
}
