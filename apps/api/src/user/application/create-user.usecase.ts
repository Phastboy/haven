import { ConflictError } from '../../shared/errors';
import type { IUserRepository } from '../domain/user.repository';
import type { User, CreateUserData } from '../domain/user.entity';

export class CreateUserUseCase {
    readonly #userRepository: IUserRepository;
  constructor(userRepository: IUserRepository) {
    this.#userRepository = userRepository;}

  async execute(data: CreateUserData): Promise<User> {
    const existing = await this.#userRepository.findByAccountId(data.accountId);
    if (existing) {
      throw new ConflictError(`A user profile for this account already exists.`);
    }
    return this.#userRepository.create(data);
  }
}
