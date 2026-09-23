import type { IAccountPlatformLinkRepository } from "../../domain/ports/IAccountPlatformLinkRepository";
import type { AccountPlatformLink } from "../../domain/account-platform-link.schema";

export class LinkPlatformUseCase {
  #platformLinkRepo: IAccountPlatformLinkRepository;
  constructor(platformLinkRepo: IAccountPlatformLinkRepository) {
    this.#platformLinkRepo = platformLinkRepo;
  }

  async execute(
    accountId: string,
    platformUserId: string,
    platform: string,
  ): Promise<AccountPlatformLink> {
    const existing = await this.#platformLinkRepo.findByAccountAndPlatform(accountId, platform);
    if (existing) {
      // Could throw an error or just return the existing link
      return existing;
    }

    return await this.#platformLinkRepo.create({
      accountId,
      platformUserId,
      platform,
    });
  }
}
