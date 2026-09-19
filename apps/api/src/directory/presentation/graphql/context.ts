import { IDirectoryRepository } from "../../domain/directory.repository";
import { DirectoryUser } from "../../domain/directory.schema";
import DataLoader from "dataloader";

export interface GraphQLContext {
  repository: IDirectoryRepository;
  userLoader: DataLoader<string, DirectoryUser | null>;
}
