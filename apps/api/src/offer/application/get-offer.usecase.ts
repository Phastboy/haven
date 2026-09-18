import { IOfferRepository } from '../domain/offer.repository';
import { Offer } from '../domain/offer.schema';
import { OfferNotFoundError } from '../domain/errors';

export class GetOfferUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  async execute(offerId: string): Promise<Offer> {
    const existing = await this.repository.findById(offerId);
    if (!existing) {
      throw new OfferNotFoundError(offerId);
    }
    return existing;
  }
}
