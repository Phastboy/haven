import type { Offer, CreateOfferData, UpdateOfferData } from "./offer.schema";

export interface IOfferRepository {
  create(userId: string, data: CreateOfferData): Promise<Offer>;
  findById(id: string): Promise<Offer | null>;
  findByUserId(userId: string, limit?: number, offset?: number): Promise<Offer[]>;
  /** Returns only non-ARCHIVED offers — for the public listing. */
  findActiveByUserId(userId: string, limit?: number, offset?: number): Promise<Offer[]>;
  update(id: string, data: UpdateOfferData): Promise<Offer | null>;
  delete(id: string): Promise<boolean>;
}
