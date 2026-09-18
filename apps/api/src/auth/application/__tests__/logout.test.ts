// oxlint-disable typescript/no-explicit-any
import { expect, test, describe, mock } from 'bun:test';
import { LogoutUseCase } from '../use-cases/logout.use-case';
import { TokenService } from '../../infrastructure/services/token.service';

describe('LogoutUseCase', () => {
  const mockSessionRepo = {
    deleteByToken: mock(),
    create: mock(),
    findByToken: mock(),
    deleteExpired: mock(),
  };

  const tokenService = new TokenService();

  const useCase = new LogoutUseCase(
    mockSessionRepo as any,
    tokenService
  );

  test('should delete session by hashed token', async () => {
    mockSessionRepo.deleteByToken.mockResolvedValueOnce(undefined);
    
    await useCase.execute('raw-token');
    
    expect(mockSessionRepo.deleteByToken).toHaveBeenCalled();
    const hashArg = mockSessionRepo.deleteByToken.mock.calls[0]![0];
    expect(hashArg).not.toBe('raw-token');
    expect(hashArg).toBe(tokenService.hash('raw-token'));
  });
});
