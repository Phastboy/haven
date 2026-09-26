import { describe, expect, test } from "bun:test";
import { loadConfig } from "../config";

describe("loadConfig", () => {
  const validBaseEnv = {
    DATABASE_URL: "postgres://user:pass@localhost:5432/db",
    TOKEN_SECRET: "1234567890123456",
    MAGIC_LINK_BASE_URL: "http://localhost:4200",
  };

  test("should load valid configuration with defaults", () => {
    const config = loadConfig(validBaseEnv);
    expect(config.DATABASE_URL).toBe(validBaseEnv.DATABASE_URL);
    expect(config.PORT).toBe(3000); // default
    expect(config.HOST).toBe("0.0.0.0"); // default
    expect(config.NODE_ENV).toBe("development"); // default
  });

  test("should override defaults when provided", () => {
    const config = loadConfig({
      ...validBaseEnv,
      PORT: "4000",
      HOST: "127.0.0.1",
      NODE_ENV: "production",
      ENABLE_CONSOLE_EMAIL: "true",
    });
    expect(config.PORT).toBe(4000);
    expect(config.HOST).toBe("127.0.0.1");
    expect(config.NODE_ENV).toBe("production");
    expect(config.ENABLE_CONSOLE_EMAIL).toBe(true);
  });

  test("should throw if required variables are missing", () => {
    const envWithoutDb = { ...validBaseEnv };
    delete (envWithoutDb as any).DATABASE_URL;

    expect(() => loadConfig(envWithoutDb)).toThrow(/DATABASE_URL/);
  });

  test("should throw if provided variables are malformed", () => {
    const envWithBadPort = { ...validBaseEnv, PORT: "not-a-number" };
    expect(() => loadConfig(envWithBadPort)).toThrow(/PORT/);
  });
});
