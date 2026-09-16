import { Account, CreateAccountDTO } from '../account.schema';

export interface IAccountRepository {
  create(data: CreateAccountDTO): Promise<Account>;
  findById(id: string): Promise<Account | null>;
  findByEmail(email: string): Promise<Account | null>;
  markEmailVerified(id: string): Promise<void>;
}
