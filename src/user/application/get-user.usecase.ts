import { NotFoundError } from '../../shared/errors';
import type { IUserRepository } from '../domain/user.repository';
import type { User } from '../domain/user.entity';

export class GetUserUseCase {
  constructor(private readonly userRepository: IUserRepository) {}

  async execute(id: string): Promise<User> {
    const user = await this.userRepository.findById(id);
    if (!user) {
      throw new NotFoundError('User', id);
    }
    return user;
  }
}
