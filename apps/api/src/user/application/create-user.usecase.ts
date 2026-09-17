import { ConflictError } from '../../shared/errors';
import type { IUserRepository } from '../domain/user.repository';
import type { User, CreateUserData } from '../domain/user.entity';

export class CreateUserUseCase {
  constructor(private readonly userRepository: IUserRepository) {}

  async execute(data: CreateUserData): Promise<User> {
    const existing = await this.userRepository.findByEmail(data.email);
    if (existing) {
      throw new ConflictError(`A user with email "${data.email}" already exists.`);
    }
    return this.userRepository.create(data);
  }
}
