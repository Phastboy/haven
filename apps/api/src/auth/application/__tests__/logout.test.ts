import { config } from "../../../config";
import { expect, test, describe, mock } from "bun:test";
import type { ISessionRepository } from "../../domain/ports/ISessionRepository";
import { LogoutUseCase } from "../use-cases/logout.use-case";
import { TokenService } from "../../infrastructure/services/token.service";

describe("LogoutUseCase", () => {
  const mockSessionRepo = {
    deleteByToken: mock(),
    create: mock(),
    findByToken: mock(),
    deleteExpired: mock(),
  };

  const tokenService = new TokenService(config);

  const useCase = new LogoutUseCase(mockSessionRepo as unknown as ISessionRepository, tokenService);

  test("should delete session by hashed token", async () => {
    mockSessionRepo.deleteByToken.mockResolvedValueOnce(undefined);

    await useCase.execute("raw-token");

    expect(mockSessionRepo.deleteByToken).toHaveBeenCalled();
    const hashArg = mockSessionRepo.deleteByToken.mock.calls[0]![0];
    expect(hashArg).not.toBe("raw-token");
    expect(hashArg).toBe(tokenService.hash("raw-token"));
  });
});
