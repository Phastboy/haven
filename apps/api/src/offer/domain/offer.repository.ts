import { Offer, CreateOfferData, UpdateOfferData } from "./offer.schema";

export interface IOfferRepository {
  create(userId: string, data: CreateOfferData): Promise<Offer>;
  findById(id: string): Promise<Offer | null>;
  findByUserId(userId: string): Promise<Offer[]>;
  /** Returns only non-ARCHIVED offers — for the public listing. */
  findActiveByUserId(userId: string): Promise<Offer[]>;
  update(id: string, data: UpdateOfferData): Promise<Offer | null>;
  delete(id: string): Promise<boolean>;
}
