import { IAccountRepository } from '../../domain/ports/IAccountRepository';
import { IOAuthCredentialRepository } from '../../domain/ports/IOAuthCredentialRepository';
import { ISessionRepository } from '../../domain/ports/ISessionRepository';
import { IGoogleTokenService } from '../../domain/ports/IGoogleTokenService';
import { TokenService } from '../../infrastructure/services/token.service';
import { Session } from '../../domain/session.schema';

export class LoginWithGoogleUseCase {
  constructor(
    private accountRepo: IAccountRepository,
    private oauthRepo: IOAuthCredentialRepository,
    private sessionRepo: ISessionRepository,
    private googleService: IGoogleTokenService,
    private tokenService: TokenService
  ) {}

  async execute(idToken: string, userAgent?: string, ipAddress?: string): Promise<Session> {
    const claims = await this.googleService.verify(idToken);
    
    let account = await this.accountRepo.findByEmail(claims.email);
    if (!account) {
      account = await this.accountRepo.create({
        email: claims.email,
        emailVerified: claims.email_verified,
      });
    } else if (claims.email_verified && !account.emailVerified) {
      await this.accountRepo.markEmailVerified(account.id);
    }

    let credential = await this.oauthRepo.findByProvider('GOOGLE', claims.sub);
    if (!credential) {
      await this.oauthRepo.create({
        accountId: account.id,
        provider: 'GOOGLE',
        providerUserId: claims.sub,
        accessToken: idToken, // Storing idToken as access token for simplicity, we don't request offline access now
      });
    } else {
      await this.oauthRepo.updateTokens(credential.id, idToken);
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

    return {
      ...session,
      token: sessionRawToken,
    };
  }
}
