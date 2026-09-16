import type { User, CreateUserData, UpdateUserData } from './user.entity';

/**
 * The contract any data source must fulfill to serve the User use cases.
 * The infrastructure layer implements this; the application layer depends on it.
 */
export interface IUserRepository {
  findById(id: string): Promise<User | null>;
  findByEmail(email: string): Promise<User | null>;
  findAll(): Promise<User[]>;
  create(data: CreateUserData): Promise<User>;
  update(id: string, data: UpdateUserData): Promise<User>;
  delete(id: string): Promise<void>;
}
