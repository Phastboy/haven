import { expect, test, describe } from 'bun:test';
import { TokenService } from '../token.service';

describe('TokenService', () => {
  const tokenService = new TokenService();

  test('generate returns a hex string of correct length', () => {
    const token = tokenService.generate(32);
    expect(typeof token).toBe('string');
    expect(token.length).toBe(64); // 32 bytes hex encoded is 64 characters
  });

  test('hash is deterministic and changes with different tokens', () => {
    const hash1 = tokenService.hash('token1');
    const hash1_again = tokenService.hash('token1');
    const hash2 = tokenService.hash('token2');

    expect(hash1).toBe(hash1_again);
    expect(hash1).not.toBe(hash2);
    expect(typeof hash1).toBe('string');
    expect(hash1.length).toBe(64);
  });
});
