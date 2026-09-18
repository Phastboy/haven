// oxlint-disable typescript/no-explicit-any
import { expect, test, describe, mock } from "bun:test";
import { RequestMagicLinkUseCase } from "../use-cases/request-magic-link.use-case";
import { TokenService } from "../../infrastructure/services/token.service";

describe("RequestMagicLinkUseCase", () => {
  const mockMagicLinkRepo = {
    create: mock(),
    findByToken: mock(),
    markUsed: mock(),
  };

  const mockEmailService = {
    send: mock(),
  };

  const tokenService = new TokenService();

  const useCase = new RequestMagicLinkUseCase(
    mockMagicLinkRepo as any,
    mockEmailService as any,
    tokenService,
  );

  test("should generate a token, save to repo, and send email", async () => {
    mockMagicLinkRepo.create.mockResolvedValueOnce({});
    mockEmailService.send.mockResolvedValueOnce(undefined);

    await useCase.execute("test@example.com");

    expect(mockMagicLinkRepo.create).toHaveBeenCalled();
    const createCall = mockMagicLinkRepo.create.mock.calls[0]![0];
    expect(createCall.email).toBe("test@example.com");
    expect(createCall.token).toBeDefined();

    expect(mockEmailService.send).toHaveBeenCalled();
    const sendCall = mockEmailService.send.mock.calls[0]!;
    expect(sendCall[0]).toBe("test@example.com");
    expect(sendCall[1]).toBe("Your Magic Login Link");
    expect(sendCall[2]).toContain("http://localhost:4200/auth/magic-login");
  });
});
