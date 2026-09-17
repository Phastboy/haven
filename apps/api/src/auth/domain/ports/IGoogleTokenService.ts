export interface GoogleClaims {
  sub: string;
  email: string;
  email_verified: boolean;
  name?: string;
  picture?: string;
}

export interface IGoogleTokenService {
  verify(idToken: string): Promise<GoogleClaims>;
}
