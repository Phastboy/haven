import { NotFoundError } from '../../shared/errors';
import type { IUserRepository } from '../domain/user.repository';

export class DeleteUserUseCase {
    readonly #userRepository: IUserRepository;
  constructor(userRepository: IUserRepository) {
    this.#userRepository = userRepository;}

  async execute(id: string): Promise<void> {
    const existing = await this.#userRepository.findById(id);
    if (!existing) {
      throw new NotFoundError('User', id);
    }
    return this.#userRepository.delete(id);
  }
}
