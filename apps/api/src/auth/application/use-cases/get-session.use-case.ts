import { ISessionRepository } from '../../domain/ports/ISessionRepository';
import { TokenService } from '../../infrastructure/services/token.service';
import { SessionWithAccount } from '../../domain/session.schema';
import { UnauthorizedError } from '../../domain/errors';

export class GetSessionUseCase {
  readonly #sessionRepo: ISessionRepository;
  readonly #tokenService: TokenService;

  constructor(
    sessionRepo: ISessionRepository,
    tokenService: TokenService
  ) {
    this.#sessionRepo = sessionRepo;
    this.#tokenService = tokenService;
  }

  async execute(rawToken: string): Promise<SessionWithAccount> {
    const hashedToken = this.#tokenService.hash(rawToken);
    const session = await this.#sessionRepo.findByToken(hashedToken);
    
    if (!session) {
      throw new UnauthorizedError('Session not found or invalid');
    }
    
    if (new Date(session.expiresAt) < new Date()) {
      await this.#sessionRepo.deleteByToken(hashedToken);
      throw new UnauthorizedError('Session expired');
    }
    
    return session;
  }
}
