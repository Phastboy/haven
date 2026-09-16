import { IMagicLinkRepository } from '../../domain/ports/IMagicLinkRepository';
import { IAccountRepository } from '../../domain/ports/IAccountRepository';
import { ISessionRepository } from '../../domain/ports/ISessionRepository';
import { TokenService } from '../../infrastructure/services/token.service';
import { InvalidTokenError, ExpiredTokenError } from '../../domain/errors';
import { Session } from '../../domain/session.schema';

export class VerifyMagicLinkUseCase {
  constructor(
    private magicLinkRepo: IMagicLinkRepository,
    private accountRepo: IAccountRepository,
    private sessionRepo: ISessionRepository,
    private tokenService: TokenService
  ) {}

  async execute(rawToken: string, userAgent?: string, ipAddress?: string): Promise<Session> {
    const hashedToken = this.tokenService.hash(rawToken);
    const magicLink = await this.magicLinkRepo.findByToken(hashedToken);

    if (!magicLink || magicLink.usedAt) {
      throw new InvalidTokenError();
    }

    if (new Date(magicLink.expiresAt) < new Date()) {
      throw new ExpiredTokenError();
    }

    await this.magicLinkRepo.markUsed(magicLink.id, new Date().toISOString());

    let account = await this.accountRepo.findByEmail(magicLink.email);
    if (!account) {
      account = await this.accountRepo.create({
        email: magicLink.email,
        emailVerified: true,
      });
    } else if (!account.emailVerified) {
      await this.accountRepo.markEmailVerified(account.id);
    }

    const sessionRawToken = this.tokenService.generate(64);
    const sessionHashedToken = this.tokenService.hash(sessionRawToken);

    const ttlDays = Number(process.env['SESSION_TTL_DAYS']) || 30;
    const sessionExpiresAt = new Date(Date.now() + ttlDays * 24 * 60 * 60 * 1000).toISOString();

    const session = await this.sessionRepo.create({
      accountId: account.id,
      token: sessionHashedToken,
      expiresAt: sessionExpiresAt,
      ...(userAgent ? { userAgent } : {}),
      ...(ipAddress ? { ipAddress } : {}),
    });

    // Return session but override the hashed token with the raw token so the client can store it
    return {
      ...session,
      token: sessionRawToken,
    };
  }
}
