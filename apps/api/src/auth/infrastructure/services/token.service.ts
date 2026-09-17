import * as crypto from 'crypto';

export class TokenService {
  private readonly secret: string;

  constructor() {
    this.secret = process.env['TOKEN_SECRET'] || '';
    if (!this.secret) {
      console.warn('WARNING: TOKEN_SECRET is not set. Token hashing will be insecure.');
    }
  }

  generate(bytes: number = 32): string {
    return crypto.randomBytes(bytes).toString('hex');
  }

  hash(token: string): string {
    if (!this.secret) {
      // Fallback for tests/dev if forgot to set
      return crypto.createHash('sha256').update(token).digest('hex');
    }
    return crypto.createHmac('sha256', this.secret).update(token).digest('hex');
  }
}

export const tokenService = new TokenService();
