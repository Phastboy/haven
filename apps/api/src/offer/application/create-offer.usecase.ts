import type { IOfferRepository } from "../domain/offer.repository";
import type { CreateOfferData, Offer } from "../domain/offer.schema";

export class CreateOfferUseCase {
  constructor(private readonly repository: IOfferRepository) {}

  async execute(userId: string, data: CreateOfferData): Promise<Offer> {
    return this.repository.create(userId, data);
  }
}
