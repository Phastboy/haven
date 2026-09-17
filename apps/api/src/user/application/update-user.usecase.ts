import { NotFoundError } from '../../shared/errors';
import type { IUserRepository } from '../domain/user.repository';
import type { User, UpdateUserData } from '../domain/user.entity';

export class UpdateUserUseCase {
  constructor(private readonly userRepository: IUserRepository) {}

  async execute(id: string, data: UpdateUserData): Promise<User> {
    const existing = await this.userRepository.findById(id);
    if (!existing) {
      throw new NotFoundError('User', id);
    }
    return this.userRepository.update(id, data);
  }
}
