import { expect, test, describe, mock } from "bun:test";
import { GetSessionUseCase } from "../use-cases/get-session.use-case";
import { TokenService } from "../../infrastructure/services/token.service";
import { UnauthorizedError } from "../../domain/errors";

describe("GetSessionUseCase", () => {
  const mockSessionRepo = {
    findByToken: mock(),
    deleteByToken: mock(),
    create: mock(),
    deleteExpired: mock(),
  };

  const tokenService = new TokenService();

  const useCase = new GetSessionUseCase(
    mockSessionRepo as unknown as import("../../domain/ports/ISessionRepository").ISessionRepository,
    tokenService,
  );

  test("should throw UnauthorizedError if session not found", async () => {
    mockSessionRepo.findByToken.mockResolvedValueOnce(null);
    await expect(useCase.execute("invalid-token")).rejects.toThrow(UnauthorizedError);
  });

  test("should throw UnauthorizedError and delete session if expired", async () => {
    const pastDate = new Date(Date.now() - 10000).toISOString();
    mockSessionRepo.findByToken.mockResolvedValueOnce({
      id: "sess-1",
      token: "hashed",
      expiresAt: pastDate,
      account: { id: "acc-1" },
    });
    mockSessionRepo.deleteByToken.mockResolvedValueOnce(undefined);

    await expect(useCase.execute("expired-token")).rejects.toThrow(UnauthorizedError);
    expect(mockSessionRepo.deleteByToken).toHaveBeenCalled();
  });

  test("should return session and account if valid", async () => {
    const futureDate = new Date(Date.now() + 10000).toISOString();
    const mockSession = {
      id: "sess-1",
      token: "hashed",
      expiresAt: futureDate,
      account: { id: "acc-1", email: "test@example.com" },
    };
    mockSessionRepo.findByToken.mockResolvedValueOnce(mockSession);

    const result = await useCase.execute("valid-token");

    expect(result.id).toBe("sess-1");
    expect(result.account.id).toBe("acc-1");
  });
});
