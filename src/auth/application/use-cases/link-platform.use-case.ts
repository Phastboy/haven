import { IAccountPlatformLinkRepository } from '../../domain/ports/IAccountPlatformLinkRepository';
import { AccountPlatformLink } from '../../domain/account-platform-link.schema';

export class LinkPlatformUseCase {
  constructor(private platformLinkRepo: IAccountPlatformLinkRepository) {}

  async execute(accountId: string, platformUserId: string, platform: string): Promise<AccountPlatformLink> {
    const existing = await this.platformLinkRepo.findByAccountAndPlatform(accountId, platform);
    if (existing) {
      // Could throw an error or just return the existing link
      return existing;
    }

    return await this.platformLinkRepo.create({
      accountId,
      platformUserId,
      platform,
    });
  }
}
