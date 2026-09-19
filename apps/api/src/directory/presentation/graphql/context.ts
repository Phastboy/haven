import {
  IDirectoryRepository,
  DirectoryOffer,
  DirectoryUser,
} from "../../domain/directory.repository";
import DataLoader from "dataloader";

export interface GraphQLContext {
  repository: IDirectoryRepository;
  userLoader: DataLoader<string, DirectoryUser | null>;
}
