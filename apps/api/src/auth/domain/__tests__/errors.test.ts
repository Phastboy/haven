// oxlint-disable typescript/no-explicit-any
import { expect, test, describe } from "bun:test";
import {
  AuthError,
  UnauthorizedError,
  InvalidTokenError,
  ExpiredTokenError,
  AccountNotFoundError,
} from "../errors";

describe("Auth Domain Errors", () => {
  test("AuthError has correct name and inheritance", () => {
    const error = new AuthError("Test error");
    expect(error.name).toBe("AuthError");
    expect(error.message).toBe("Test error");
    expect(error instanceof Error).toBe(true);
  });

  test("UnauthorizedError has correct message", () => {
    const error = new UnauthorizedError();
    expect(error.name).toBe("UnauthorizedError");
    expect(error.message).toBe("Unauthorized access");
    expect(error instanceof AuthError).toBe(true);
  });

  test("InvalidTokenError has correct message", () => {
    const error = new InvalidTokenError();
    expect(error.name).toBe("InvalidTokenError");
    expect(error.message).toBe("Invalid or used token");
    expect(error instanceof AuthError).toBe(true);
  });

  test("ExpiredTokenError has correct message", () => {
    const error = new ExpiredTokenError();
    expect(error.name).toBe("ExpiredTokenError");
    expect(error.message).toBe("Token has expired");
    expect(error instanceof AuthError).toBe(true);
  });

  test("AccountNotFoundError has correct message", () => {
    const error = new AccountNotFoundError();
    expect(error.name).toBe("AccountNotFoundError");
    expect(error.message).toBe("Account not found");
    expect(error instanceof AuthError).toBe(true);
  });
});
