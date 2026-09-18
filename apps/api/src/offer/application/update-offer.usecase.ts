import { IOfferRepository } from '../domain/offer.repository';
import { UpdateOfferData, Offer } from '../domain/offer.schema';
import { OfferNotFoundError, UnauthorizedOfferActionError } from '../domain/errors';

export class UpdateOfferUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  async execute(userId: string, offerId: string, data: UpdateOfferData): Promise<Offer> {
    const existing = await this.repository.findById(offerId);
    if (!existing) {
      throw new OfferNotFoundError(offerId);
    }

    if (existing.userId !== userId) {
      throw new UnauthorizedOfferActionError();
    }

    const updated = await this.repository.update(offerId, data);
    if (!updated) {
      throw new OfferNotFoundError(offerId); // Should not happen, but for safety
    }

    return updated;
  }
}
