import type { IDirectoryRepository } from "../../domain/directory.repository";
import type { DirectoryUser } from "../../domain/directory.schema";
import type DataLoader from "dataloader";

export interface GraphQLContext {
  repository: IDirectoryRepository;
  userLoader: DataLoader<string, DirectoryUser | null>;
}
