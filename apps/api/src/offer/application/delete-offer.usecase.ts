import type { IOfferRepository } from "../domain/offer.repository";
import { OfferNotFoundError, UnauthorizedOfferActionError } from "../domain/errors";

export class DeleteOfferUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  async execute(userId: string, offerId: string): Promise<void> {
    const existing = await this.repository.findById(offerId);
    if (!existing) {
      throw new OfferNotFoundError();
    }

    if (existing.userId !== userId) {
      throw new UnauthorizedOfferActionError();
    }

    await this.repository.delete(offerId);
  }
}
