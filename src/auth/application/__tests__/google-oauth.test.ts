import { expect, test, describe, mock } from 'bun:test';
import { LoginWithGoogleUseCase } from '../use-cases/login-with-google.use-case';
import { TokenService } from '../../infrastructure/services/token.service';

describe('LoginWithGoogleUseCase', () => {
  const mockAccountRepo = {
    create: mock(),
    findById: mock(),
    findByEmail: mock(),
    markEmailVerified: mock(),
  };

  const mockOauthRepo = {
    create: mock(),
    findByProvider: mock(),
    updateTokens: mock(),
  };

  const mockSessionRepo = {
    create: mock(),
    findByToken: mock(),
    deleteByToken: mock(),
    deleteExpired: mock(),
  };

  const mockGoogleService = {
    verify: mock(),
  };

  const tokenService = new TokenService();

  const useCase = new LoginWithGoogleUseCase(
    mockAccountRepo as any,
    mockOauthRepo as any,
    mockSessionRepo as any,
    mockGoogleService as any,
    tokenService
  );

  test('should create account and credential if new, and return session', async () => {
    mockGoogleService.verify.mockResolvedValueOnce({
      sub: 'google-sub-1',
      email: 'google@example.com',
      email_verified: true,
      name: 'Google User',
    });

    mockAccountRepo.findByEmail.mockResolvedValueOnce(null);
    mockAccountRepo.create.mockResolvedValueOnce({
      id: 'acc-google-1',
      email: 'google@example.com',
      emailVerified: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });

    mockOauthRepo.findByProvider.mockResolvedValueOnce(null);
    mockOauthRepo.create.mockResolvedValueOnce({
      id: 'cred-1',
      accountId: 'acc-google-1',
      provider: 'GOOGLE',
      providerUserId: 'google-sub-1',
      accessToken: 'id-token-xyz',
    });

    mockSessionRepo.create.mockResolvedValueOnce({
      id: 'sess-google-1',
      accountId: 'acc-google-1',
      token: 'hashed-session',
      expiresAt: new Date().toISOString(),
      userAgent: undefined,
      ipAddress: undefined,
      createdAt: new Date().toISOString(),
    });

    const session = await useCase.execute('id-token-xyz');

    expect(mockGoogleService.verify).toHaveBeenCalledWith('id-token-xyz');
    expect(mockAccountRepo.create).toHaveBeenCalled();
    expect(mockOauthRepo.create).toHaveBeenCalled();
    expect(mockSessionRepo.create).toHaveBeenCalled();
    expect(session.accountId).toBe('acc-google-1');
  });

  test('should reuse existing account and credential if found', async () => {
    mockGoogleService.verify.mockResolvedValueOnce({
      sub: 'google-sub-1',
      email: 'google@example.com',
      email_verified: true,
      name: 'Google User',
    });

    mockAccountRepo.findByEmail.mockResolvedValueOnce({
      id: 'acc-google-1',
      email: 'google@example.com',
      emailVerified: true,
    });

    mockOauthRepo.findByProvider.mockResolvedValueOnce({
      id: 'cred-1',
      accountId: 'acc-google-1',
      provider: 'GOOGLE',
      providerUserId: 'google-sub-1',
      accessToken: 'old-token',
    });
    mockOauthRepo.updateTokens.mockResolvedValueOnce(undefined);

    mockSessionRepo.create.mockResolvedValueOnce({
      id: 'sess-google-2',
      accountId: 'acc-google-1',
      token: 'hashed-session-2',
      expiresAt: new Date().toISOString(),
    });

    const session = await useCase.execute('new-id-token');

    expect(mockGoogleService.verify).toHaveBeenCalledWith('new-id-token');
    expect(mockAccountRepo.findByEmail).toHaveBeenCalledWith('google@example.com');
    expect(mockOauthRepo.updateTokens).toHaveBeenCalledWith('cred-1', 'new-id-token');
    expect(mockSessionRepo.create).toHaveBeenCalled();
    expect(session.accountId).toBe('acc-google-1');
  });
});
