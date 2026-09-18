import { NotFoundError } from '../../shared/errors';
import type { IUserRepository } from '../domain/user.repository';
import type { User, UpdateUserData } from '../domain/user.entity';

export class UpdateUserUseCase {
    readonly #userRepository: IUserRepository;
  constructor(userRepository: IUserRepository) {
    this.#userRepository = userRepository;}

  async execute(accountId: string, data: UpdateUserData): Promise<User> {
    const existing = await this.#userRepository.findByAccountId(accountId);
    if (!existing) {
      throw new NotFoundError('User', accountId);
    }
    return this.#userRepository.update(existing.id, data);
  }
}
