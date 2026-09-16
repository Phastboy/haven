import type { IUserRepository } from '../domain/user.repository';
import type { User } from '../domain/user.entity';

export class ListUsersUseCase {
  constructor(private readonly userRepository: IUserRepository) {}

  async execute(): Promise<User[]> {
    return this.userRepository.findAll();
  }
}
