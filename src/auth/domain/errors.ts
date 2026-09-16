export class AuthError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AuthError';
  }
}

export class InvalidTokenError extends AuthError {
  constructor(message: string = 'Invalid or used token') {
    super(message);
    this.name = 'InvalidTokenError';
  }
}

export class ExpiredTokenError extends AuthError {
  constructor(message: string = 'Token has expired') {
    super(message);
    this.name = 'ExpiredTokenError';
  }
}

export class UnauthorizedError extends AuthError {
  constructor(message: string = 'Unauthorized access') {
    super(message);
    this.name = 'UnauthorizedError';
  }
}

export class AccountNotFoundError extends AuthError {
  constructor(message: string = 'Account not found') {
    super(message);
    this.name = 'AccountNotFoundError';
  }
}
