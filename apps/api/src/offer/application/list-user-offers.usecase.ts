import type { IOfferRepository } from "../domain/offer.repository";
import type { Offer } from "../domain/offer.schema";

export class ListUserOffersUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  /**
   * @param userId     The profile whose offers to list.
   * @param requesterId  The account making the request (undefined = unauthenticated).
   *                   If requesterId === userId the caller is the owner and sees all statuses.
   */
  async execute(userId: string, requesterId?: string): Promise<Offer[]> {
    if (requesterId === userId) {
      return this.repository.findByUserId(userId);
    }
    return this.repository.findActiveByUserId(userId);
  }
}
