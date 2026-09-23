import { AccountPlatformLink } from "../account-platform-link.schema";

export interface CreateAccountPlatformLinkDTO {
  accountId: string;
  platformUserId: string;
  platform: string;
}

export interface IAccountPlatformLinkRepository {
  create(data: CreateAccountPlatformLinkDTO): Promise<AccountPlatformLink>;
  findByAccountAndPlatform(
    accountId: string,
    platform: string,
  ): Promise<AccountPlatformLink | null>;
}
