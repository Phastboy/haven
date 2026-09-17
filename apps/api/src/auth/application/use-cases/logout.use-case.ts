import { ISessionRepository } from '../../domain/ports/ISessionRepository';
import { TokenService } from '../../infrastructure/services/token.service';

export class LogoutUseCase {
  constructor(
    private sessionRepo: ISessionRepository,
    private tokenService: TokenService
  ) {}

  async execute(rawToken: string): Promise<void> {
    const hashedToken = this.tokenService.hash(rawToken);
    await this.sessionRepo.deleteByToken(hashedToken);
  }
}
