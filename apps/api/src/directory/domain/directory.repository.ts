import type { DirectoryOffer, DirectoryUser } from "./directory.schema";

export interface IDirectoryRepository {
  getActiveOffers(limit?: number, offset?: number): Promise<DirectoryOffer[]>;
  getUsersByIds(userIds: string[]): Promise<DirectoryUser[]>;
}
