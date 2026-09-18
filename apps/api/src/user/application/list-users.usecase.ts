import type { IUserRepository } from '../domain/user.repository';
import type { User } from '../domain/user.entity';

export class ListUsersUseCase {
    readonly #userRepository: IUserRepository;
  constructor(userRepository: IUserRepository) {
    this.#userRepository = userRepository;}

  async execute(): Promise<User[]> {
    return this.#userRepository.findAll();
  }
}
