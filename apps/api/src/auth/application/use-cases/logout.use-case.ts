import { ISessionRepository } from '../../domain/ports/ISessionRepository';
import { TokenService } from '../../infrastructure/services/token.service';

export class LogoutUseCase {
  readonly #sessionRepo: ISessionRepository;
  readonly #tokenService: TokenService;

  constructor(
    sessionRepo: ISessionRepository,
    tokenService: TokenService
  ) {
    this.#sessionRepo = sessionRepo;
    this.#tokenService = tokenService;
  }

  async execute(rawToken: string): Promise<void> {
    const hashedToken = this.#tokenService.hash(rawToken);
    await this.#sessionRepo.deleteByToken(hashedToken);
  }
}
