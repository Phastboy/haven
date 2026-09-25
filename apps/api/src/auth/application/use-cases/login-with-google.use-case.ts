import type { IAccountRepository } from "../../domain/ports/IAccountRepository";
import type { IOAuthCredentialRepository } from "../../domain/ports/IOAuthCredentialRepository";
import type { ISessionRepository } from "../../domain/ports/ISessionRepository";
import type { IGoogleTokenService } from "../../domain/ports/IGoogleTokenService";
import type { TokenService } from "../../infrastructure/services/token.service";
import type { Session } from "../../domain/session.schema";
import type { IProfileCreator } from "../../domain/ports/IProfileCreator";
import type { Config } from "../../../config";

export class LoginWithGoogleUseCase {
  readonly #accountRepo: IAccountRepository;
  readonly #oauthRepo: IOAuthCredentialRepository;
  readonly #sessionRepo: ISessionRepository;
  readonly #googleService: IGoogleTokenService;
  readonly #tokenService: TokenService;
  readonly #profileCreator: IProfileCreator;
  readonly #config: Config;

  constructor(
    accountRepo: IAccountRepository,
    oauthRepo: IOAuthCredentialRepository,
    sessionRepo: ISessionRepository,
    googleService: IGoogleTokenService,
    tokenService: TokenService,
    profileCreator: IProfileCreator,
    config: Config,
  ) {
    this.#accountRepo = accountRepo;
    this.#oauthRepo = oauthRepo;
    this.#sessionRepo = sessionRepo;
    this.#googleService = googleService;
    this.#tokenService = tokenService;
    this.#profileCreator = profileCreator;
    this.#config = config;
  }

  async execute(idToken: string, userAgent?: string, ipAddress?: string): Promise<Session> {
    const claims = await this.#googleService.verify(idToken);
    let credential = await this.#oauthRepo.findByProvider("GOOGLE", claims.sub);
    let account: { id: string; emailVerified: boolean } | null = null;

    if (credential) {
      account = await this.#accountRepo.findById(credential.accountId);
      if (!account) {
        throw new Error("Inconsistent state: OAuth credential exists but account does not");
      }
    } else {
      if (!claims.email_verified) {
        throw new Error("Email must be verified to link a new Google account");
      }
      account = await this.#accountRepo.findByEmail(claims.email);

      if (!account) {
        account = await this.#accountRepo.create({
          email: claims.email,
          emailVerified: true, // we already know it's verified here
        });
        await this.#profileCreator.createProfileForAccount(account.id);
      } else if (!account.emailVerified) {
        await this.#accountRepo.markEmailVerified(account.id);
      }

      credential = await this.#oauthRepo.create({
        accountId: account.id,
        provider: "GOOGLE",
        providerUserId: claims.sub,
        accessToken: claims.sub, // Storing sub since we don't request offline access right now
      });
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

    return {
      ...session,
      token: sessionRawToken,
    };
  }
}
