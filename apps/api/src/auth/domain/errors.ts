import { DomainError } from "../../shared/domain/errors";

export class AuthError extends DomainError {
  constructor(
    message: string,
    status: number = 401,
    type: string = "auth_error",
    title: string = "Authentication Error",
  ) {
    super(type, title, status, message);
  }
}

export class InvalidTokenError extends AuthError {
  constructor(message: string = "Invalid or used token") {
    super(message, 401, "invalid_token", "Invalid Token");
  }
}

export class ExpiredTokenError extends AuthError {
  constructor(message: string = "Token has expired") {
    super(message, 401, "expired_token", "Token Expired");
  }
}

export class UnauthorizedError extends AuthError {
  constructor(message: string = "Unauthorized access") {
    super(message, 401, "unauthorized", "Unauthorized");
  }
}

export class AccountNotFoundError extends AuthError {
  constructor(message: string = "Account not found") {
    super(message, 404, "account_not_found", "Account Not Found");
  }
}
