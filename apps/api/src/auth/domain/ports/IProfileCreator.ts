export interface IProfileCreator {
  createProfileForAccount(accountId: string): Promise<void>;
}
