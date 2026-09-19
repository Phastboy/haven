import { IOfferRepository } from "../domain/offer.repository";
import { Offer } from "../domain/offer.schema";

export class ListUserOffersUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  async execute(userId: string): Promise<Offer[]> {
    return this.repository.findByUserId(userId);
  }
}
