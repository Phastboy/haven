import type { IUserRepository } from "../domain/user.repository";
import type { User } from "../domain/user.entity";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class ListUsersUseCase {
  readonly #userRepository: IUserRepository;
  constructor(userRepository: IUserRepository) {
    this.#userRepository = userRepository;
  }

  async execute(): Promise<PaginatedResponse<User>> {
    const data = await this.#userRepository.findAll();
    return createPaginatedResponse(data, { total: data.length });
  }
}
