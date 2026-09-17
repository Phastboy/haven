import { OAuthCredential } from '../oauth-credential.schema';

export interface CreateOAuthCredentialDTO {
  accountId: string;
  provider: 'GOOGLE';
  providerUserId: string;
  accessToken: string;
  refreshToken?: string;
  tokenExpiresAt?: string;
}

export interface IOAuthCredentialRepository {
  create(data: CreateOAuthCredentialDTO): Promise<OAuthCredential>;
  findByProvider(provider: 'GOOGLE', providerUserId: string): Promise<OAuthCredential | null>;
  updateTokens(id: string, accessToken: string, refreshToken?: string, tokenExpiresAt?: string): Promise<void>;
}
