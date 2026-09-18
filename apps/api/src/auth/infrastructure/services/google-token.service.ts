import { OAuth2Client } from 'google-auth-library';
import { IGoogleTokenService, GoogleClaims } from '../../domain/ports/IGoogleTokenService';
import { UnauthorizedError } from '../../domain/errors';

export class GoogleTokenService implements IGoogleTokenService {
  private client: OAuth2Client;
  private clientId: string;

  constructor() {
    this.clientId = process.env['GOOGLE_CLIENT_ID'] || '';
    if (!this.clientId) {
      console.warn('WARNING: GOOGLE_CLIENT_ID is not set.');
    }
    this.client = new OAuth2Client(this.clientId);
  }

  async verify(idToken: string): Promise<GoogleClaims> {
    try {
      const ticket = await this.client.verifyIdToken({
        idToken,
        audience: this.clientId,
      });
      const payload = ticket.getPayload();
      
      if (!payload || !payload.sub || !payload.email) {
        throw new UnauthorizedError('Invalid Google token payload');
      }

      return {
        sub: payload.sub,
        email: payload.email,
        email_verified: payload.email_verified || false,
        ...(payload.name ? { name: payload.name } : {}),
        ...(payload.picture ? { picture: payload.picture } : {}),
      };
    } catch (error) {
      console.error('Google verifyIdToken failed:', error);
      throw new UnauthorizedError('Invalid Google token: ' + (error instanceof Error ? error.message : String(error)));
    }
  }
}
