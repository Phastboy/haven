import { expect, test, describe, mock } from 'bun:test';
import { VerifyMagicLinkUseCase } from '../use-cases/verify-magic-link.use-case';
import { InvalidTokenError, ExpiredTokenError } from '../../domain/errors';
import { TokenService } from '../../infrastructure/services/token.service';

describe('VerifyMagicLinkUseCase', () => {
  const mockMagicLinkRepo = {
    create: mock(),
    findByToken: mock(),
    markUsed: mock(),
  };

  const mockAccountRepo = {
    create: mock(),
    findById: mock(),
    findByEmail: mock(),
    markEmailVerified: mock(),
  };

  const mockSessionRepo = {
    create: mock(),
    findByToken: mock(),
    deleteByToken: mock(),
    deleteExpired: mock(),
  };

  const tokenService = new TokenService();

  const useCase = new VerifyMagicLinkUseCase(
    mockMagicLinkRepo,
    mockAccountRepo,
    mockSessionRepo,
    tokenService
  );

  test('should throw InvalidTokenError if magic link not found', async () => {
    mockMagicLinkRepo.findByToken.mockResolvedValueOnce(null);
    await expect(useCase.execute('invalid-token')).rejects.toThrow(InvalidTokenError);
  });

  test('should throw ExpiredTokenError if magic link is expired', async () => {
    const pastDate = new Date(Date.now() - 10000).toISOString();
    mockMagicLinkRepo.findByToken.mockResolvedValueOnce({
      id: 'ml-1',
      email: 'test@example.com',
      token: 'hashed',
      expiresAt: pastDate,
      createdAt: new Date().toISOString(),
    });

    await expect(useCase.execute('expired-token')).rejects.toThrow(ExpiredTokenError);
  });

  test('should verify valid token, create account if new, and return session', async () => {
    const futureDate = new Date(Date.now() + 10000).toISOString();
    mockMagicLinkRepo.findByToken.mockResolvedValueOnce({
      id: 'ml-1',
      email: 'new@example.com',
      token: 'hashed',
      expiresAt: futureDate,
      usedAt: null,
      createdAt: new Date().toISOString(),
    });

    mockMagicLinkRepo.markUsed.mockResolvedValueOnce(true);
    
    // Simulate new account
    mockAccountRepo.findByEmail.mockResolvedValueOnce(null);
    mockAccountRepo.create.mockResolvedValueOnce({
      id: 'acc-1',
      email: 'new@example.com',
      emailVerified: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });

    mockSessionRepo.create.mockResolvedValueOnce({
      id: 'sess-1',
      accountId: 'acc-1',
      token: 'hashed-session',
      expiresAt: futureDate,
      userAgent: 'test-agent',
      ipAddress: '127.0.0.1',
      createdAt: new Date().toISOString(),
    });

    const session = await useCase.execute('valid-token', 'test-agent', '127.0.0.1');

    expect(mockMagicLinkRepo.markUsed).toHaveBeenCalled();
    expect(mockAccountRepo.findByEmail).toHaveBeenCalledWith('new@example.com');
    expect(mockAccountRepo.create).toHaveBeenCalled();
    expect(mockSessionRepo.create).toHaveBeenCalled();
    expect(session.accountId).toBe('acc-1');
    expect(session.token).toBeDefined(); // Contains the raw token
  });
});
