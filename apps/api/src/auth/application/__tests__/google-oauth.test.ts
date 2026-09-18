// oxlint-disable typescript/no-explicit-any
import { expect, test, describe, mock, beforeEach } from 'bun:test';
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

  const mockProfileCreator = {
    createProfileForAccount: mock(),
  };

  const tokenService = new TokenService();

  const useCase = new LoginWithGoogleUseCase(
    mockAccountRepo as any,
    mockOauthRepo as any,
    mockSessionRepo as any,
    mockGoogleService as any,
    tokenService,
    mockProfileCreator as any
  );

  beforeEach(() => {
    mockAccountRepo.create.mockClear();
    mockAccountRepo.findById.mockClear();
    mockAccountRepo.findByEmail.mockClear();
    mockAccountRepo.markEmailVerified.mockClear();
    mockOauthRepo.create.mockClear();
    mockOauthRepo.findByProvider.mockClear();
    mockOauthRepo.updateTokens.mockClear();
    mockSessionRepo.create.mockClear();
    mockGoogleService.verify.mockClear();
    mockProfileCreator.createProfileForAccount.mockClear();
  });

  test('should create account and credential if new, and return session', async () => {
    mockGoogleService.verify.mockResolvedValueOnce({
      sub: 'google-sub-1',
      email: 'google@example.com',
      email_verified: true,
      name: 'Google User',
    });

    mockOauthRepo.findByProvider.mockResolvedValueOnce(null);
    mockAccountRepo.findByEmail.mockResolvedValueOnce(null);
    mockAccountRepo.create.mockResolvedValueOnce({
      id: 'acc-google-1',
      email: 'google@example.com',
      emailVerified: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });

    mockOauthRepo.create.mockResolvedValueOnce({
      id: 'cred-1',
      accountId: 'acc-google-1',
      provider: 'GOOGLE',
      providerUserId: 'google-sub-1',
      accessToken: 'google-sub-1', // claims.sub is stored
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
    expect(mockOauthRepo.create).toHaveBeenCalledWith({
      accountId: 'acc-google-1',
      provider: 'GOOGLE',
      providerUserId: 'google-sub-1',
      accessToken: 'google-sub-1',
    });
    expect(mockProfileCreator.createProfileForAccount).toHaveBeenCalledWith('acc-google-1');
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

    mockOauthRepo.findByProvider.mockResolvedValueOnce({
      id: 'cred-1',
      accountId: 'acc-google-1',
      provider: 'GOOGLE',
      providerUserId: 'google-sub-1',
      accessToken: 'google-sub-1',
    });

    mockAccountRepo.findById.mockResolvedValueOnce({
      id: 'acc-google-1',
      email: 'google@example.com',
      emailVerified: true,
    });

    mockSessionRepo.create.mockResolvedValueOnce({
      id: 'sess-google-2',
      accountId: 'acc-google-1',
      token: 'hashed-session-2',
      expiresAt: new Date().toISOString(),
    });

    const session = await useCase.execute('new-id-token');

    expect(mockGoogleService.verify).toHaveBeenCalledWith('new-id-token');
    expect(mockAccountRepo.findById).toHaveBeenCalledWith('acc-google-1');
    expect(mockOauthRepo.updateTokens).not.toHaveBeenCalled();
    expect(mockSessionRepo.create).toHaveBeenCalled();
    expect(session.accountId).toBe('acc-google-1');
  });

  test('should throw error if email is unverified and no existing credential is linked', async () => {
    mockGoogleService.verify.mockResolvedValueOnce({
      sub: 'google-sub-unverified',
      email: 'hacker@example.com',
      email_verified: false,
    });

    mockOauthRepo.findByProvider.mockResolvedValueOnce(null);

    await expect(useCase.execute('unverified-id-token')).rejects.toThrow('Email must be verified to link a new Google account');
    
    expect(mockAccountRepo.findByEmail).not.toHaveBeenCalled();
    expect(mockAccountRepo.create).not.toHaveBeenCalled();
    expect(mockOauthRepo.create).not.toHaveBeenCalled();
  });
});
