import { config } from "../../../config";
import { sql } from "drizzle-orm";
import { describe, expect, it, beforeAll, afterAll } from "bun:test";
import { createAuthPlugin } from "../auth.controller";
import { createDb } from "../../../database/db";
const db = createDb(config);


const mockProfileCreator = { createProfileForAccount: async () => {} };
const authController = createAuthPlugin(mockProfileCreator, config, db);
import { TokenService } from "../../infrastructure/services/token.service";
const tokenService = new TokenService(config);
import { SessionRepository } from "../../infrastructure/repositories/session.repository";
import * as crypto from "crypto";

describe("Session Middleware E2E", () => {
  let testAccountId: string;
  let validRawToken: string;
  let expiredRawToken: string;

  beforeAll(async () => {
    const accountId = crypto.randomUUID();
    testAccountId = accountId;
    await db.execute(sql`
      INSERT INTO "Account" (id, email, "emailVerified")
      VALUES (${accountId}, ${`e2e-${Date.now()}@test.com`}, true)
    `);

    const repo = new SessionRepository(db);

    validRawToken = tokenService.generate(64);
    await repo.create({
      accountId,
      token: tokenService.hash(validRawToken),
      expiresAt: new Date(Date.now() + 100000).toISOString(),
    });

    expiredRawToken = tokenService.generate(64);
    await repo.create({
      accountId,
      token: tokenService.hash(expiredRawToken),
      expiresAt: new Date(Date.now() - 100000).toISOString(),
    });
  });

  afterAll(async () => {
    await db.execute(sql`DELETE FROM "Account" WHERE id = ${testAccountId}`);
  });

  it("should successfully get the user account using a valid token", async () => {
    const req = new Request("http://localhost/auth/me", {
      headers: { Authorization: `Bearer ${validRawToken}` },
    });
    const res = await authController.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as { data?: { account: { id: string } } };
    expect(body).not.toBeNull();
    expect(body.data?.account).toBeDefined();
    expect(body.data?.account.id).toBe(testAccountId);
  });

  it("should return 401 Unauthorized if no token is provided", async () => {
    const req = new Request("http://localhost/auth/me");
    const res = await authController.handle(req);
    expect(res.status).toBe(401);
    const data = (await res.json()) as { token?: string; [key: string]: unknown };
    expect(data).toMatchObject({ detail: "Unauthorized access", status: 401 });
  });

  it("should return 401 Unauthorized if the token is invalid", async () => {
    const req = new Request("http://localhost/auth/me", {
      headers: { Authorization: "Bearer invalid_garbage_token" },
    });
    const res = await authController.handle(req);
    expect(res.status).toBe(401);
    const data = (await res.json()) as { token?: string; [key: string]: unknown };
    expect(data).toMatchObject({ detail: "Unauthorized access", status: 401 });
  });

  it("should return 401 Unauthorized if the token is expired", async () => {
    const req = new Request("http://localhost/auth/me", {
      headers: { Authorization: `Bearer ${expiredRawToken}` },
    });
    const res = await authController.handle(req);
    expect(res.status).toBe(401);
    const data = (await res.json()) as { token?: string; [key: string]: unknown };
    expect(data).toMatchObject({ detail: "Unauthorized access", status: 401 });
  });
});
