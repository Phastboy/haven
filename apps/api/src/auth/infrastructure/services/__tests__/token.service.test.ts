import { describe, expect, it } from "bun:test";
import { TokenService } from "../token.service";
import * as crypto from "crypto";

describe("TokenService", () => {
  it("should generate a 64-byte token successfully as a 128-character hex string", () => {
    const tokenService = new TokenService({ TOKEN_SECRET: "test-secret" } as any);
    const token = tokenService.generate(64);

    expect(typeof token).toBe("string");
    expect(token.length).toBe(128); // 64 bytes in hex is 128 characters
  });

  it("should consistently hash the token", () => {
    const tokenService = new TokenService({ TOKEN_SECRET: "test-secret" } as any);

    const token =
      "12158192c818405d553d888d2d629d998ef9c213c75841681ed767b9783b21dc4fc0274684eb8e85ff66cceabbe7876a2cb4a17f8241987a0e2280aa0b5ae563";
    const hash1 = tokenService.hash(token);
    const hash2 = tokenService.hash(token);

    expect(hash1).toBe(hash2);
    expect(hash1.length).toBe(64); // sha256 hex is 64 characters

    // Manual verification
    const expectedHash = crypto.createHmac("sha256", "test-secret").update(token).digest("hex");
    expect(hash1).toBe(expectedHash);
  });
});
