// oxlint-disable typescript/no-explicit-any
import { describe, expect, it, mock } from "bun:test";
import { GetSessionUseCase } from "../use-cases/get-session.use-case";
import { ISessionRepository } from "../../domain/ports/ISessionRepository";
import { UnauthorizedError } from "../../domain/errors";
import { SessionWithAccount } from "../../domain/session.schema";

describe("GetSessionUseCase", () => {
  const mockTokenService = {
    hash: mock((token: string) => `hashed_${token}`),
    generate: mock(() => "raw_token"),
  } as any;

  it("should successfully return an active session", async () => {
    const mockSession: SessionWithAccount = {
      id: "session-id",
      accountId: "account-id",
      token: "hashed_raw_token",
      expiresAt: new Date(Date.now() + 100000).toISOString(),
      createdAt: new Date().toISOString(),
      account: {
        id: "account-id",
        email: "test@test.com",
        emailVerified: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
    };

    const mockRepo = {
      findByToken: mock(async () => mockSession),
      create: mock(async () => mockSession),
      deleteByToken: mock(async () => {}),
      deleteExpired: mock(async () => {}),
    } as ISessionRepository;

    const useCase = new GetSessionUseCase(mockRepo as any, mockTokenService as any);

    const result = await useCase.execute("raw_token");

    expect(result.id).toBe("session-id");
    expect(mockTokenService.hash).toHaveBeenCalledWith("raw_token");
    expect(mockRepo.findByToken).toHaveBeenCalledWith("hashed_raw_token");
  });

  it("should throw UnauthorizedError if session does not exist", async () => {
    const mockRepo = {
      findByToken: mock(async () => null),
    } as any as ISessionRepository;

    const useCase = new GetSessionUseCase(mockRepo as any, mockTokenService as any);

    expect(useCase.execute("bad_token")).rejects.toThrow(UnauthorizedError);
  });

  it("should throw UnauthorizedError and delete session if expired", async () => {
    const mockSession: SessionWithAccount = {
      id: "session-id",
      accountId: "account-id",
      token: "hashed_expired_token",
      expiresAt: new Date(Date.now() - 100000).toISOString(), // Past date
      createdAt: new Date().toISOString(),
      account: {
        id: "account-id",
        email: "test@test.com",
        emailVerified: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
    };

    const mockRepo = {
      findByToken: mock(async () => mockSession),
      deleteByToken: mock(async () => {}),
    } as any as ISessionRepository;

    const useCase = new GetSessionUseCase(mockRepo as any, mockTokenService as any);

    expect(useCase.execute("expired_token")).rejects.toThrow(
      new UnauthorizedError("Session expired"),
    );
    expect(mockRepo.deleteByToken).toHaveBeenCalledWith("hashed_expired_token");
  });
});
