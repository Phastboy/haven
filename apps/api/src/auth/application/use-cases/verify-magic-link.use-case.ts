import type { IMagicLinkRepository } from "../../domain/ports/IMagicLinkRepository";
import type { IAccountRepository } from "../../domain/ports/IAccountRepository";
import type { ISessionRepository } from "../../domain/ports/ISessionRepository";
import type { TokenService } from "../../infrastructure/services/token.service";
import { InvalidTokenError, ExpiredTokenError } from "../../domain/errors";
import type { Session } from "../../domain/session.schema";
import type { IProfileCreator } from "../../domain/ports/IProfileCreator";
import type { Config } from "../../../config";

export class VerifyMagicLinkUseCase {
  readonly #magicLinkRepo: IMagicLinkRepository;
  readonly #accountRepo: IAccountRepository;
  readonly #sessionRepo: ISessionRepository;
  readonly #tokenService: TokenService;
  readonly #profileCreator: IProfileCreator;
  readonly #config: Config;

  constructor(
    magicLinkRepo: IMagicLinkRepository,
    accountRepo: IAccountRepository,
    sessionRepo: ISessionRepository,
    tokenService: TokenService,
    profileCreator: IProfileCreator,
    config: Config,
  ) {
    this.#magicLinkRepo = magicLinkRepo;
    this.#accountRepo = accountRepo;
    this.#sessionRepo = sessionRepo;
    this.#tokenService = tokenService;
    this.#profileCreator = profileCreator;
    this.#config = config;
  }

  async execute(rawToken: string, userAgent?: string, ipAddress?: string): Promise<Session> {
    const hashedToken = this.#tokenService.hash(rawToken);
    const magicLink = await this.#magicLinkRepo.findByToken(hashedToken);

    if (!magicLink || magicLink.usedAt) {
      throw new InvalidTokenError();
    }

    if (new Date(magicLink.expiresAt) < new Date()) {
      throw new ExpiredTokenError();
    }

    const marked = await this.#magicLinkRepo.markUsed(magicLink.id, new Date().toISOString());
    if (!marked) {
      throw new InvalidTokenError();
    }

    let account = await this.#accountRepo.findByEmail(magicLink.email);
    if (!account) {
      account = await this.#accountRepo.create({
        email: magicLink.email,
        emailVerified: true,
      });
      await this.#profileCreator.createProfileForAccount(account.id);
    } else if (!account.emailVerified) {
      await this.#accountRepo.markEmailVerified(account.id);
    }

    const sessionRawToken = this.#tokenService.generate(64);
    const sessionHashedToken = this.#tokenService.hash(sessionRawToken);

    const ttlDays = this.#config.SESSION_TTL_DAYS;
    const sessionExpiresAt = new Date(Date.now() + ttlDays * 24 * 60 * 60 * 1000).toISOString();

    const session = await this.#sessionRepo.create({
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
