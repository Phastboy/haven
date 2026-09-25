import { Type } from "@sinclair/typebox";
import { Value } from "@sinclair/typebox/value";

export const ConfigSchema = Type.Object({
  NODE_ENV: Type.Union(
    [Type.Literal("development"), Type.Literal("test"), Type.Literal("production")],
    { default: "development" }
  ),
  PORT: Type.Number({ default: 3000 }),
  HOST: Type.String({ default: "0.0.0.0" }),
  DATABASE_URL: Type.String({ minLength: 1, description: "Postgres connection string" }),
  WEB_ORIGIN: Type.String({ default: "http://localhost:4200" }),
  TOKEN_SECRET: Type.String({ minLength: 16 }),
  SESSION_TTL_DAYS: Type.Number({ default: 30 }),
  MAGIC_LINK_TTL_MINUTES: Type.Number({ default: 15 }),
  MAGIC_LINK_BASE_URL: Type.String({ minLength: 1 }),
  GOOGLE_CLIENT_ID: Type.Optional(Type.String()),
  
  // Email/SMTP Configuration
  ENABLE_CONSOLE_EMAIL: Type.Boolean({ default: false }),
  EMAIL_FROM: Type.String({ default: "noreply@haven.app" }),
  SMTP_HOST: Type.Optional(Type.String()),
  SMTP_PORT: Type.Optional(Type.Number()),
  SMTP_USER: Type.Optional(Type.String()),
  SMTP_PASS: Type.Optional(Type.String()),
});

export type Config = typeof ConfigSchema.static;

/**
 * Parses and validates environment variables against ConfigSchema.
 * Throws a detailed error if required variables are missing or malformed.
 */
export function loadConfig(env: Record<string, string | undefined> = process.env): Config {
  // Coerce specific types from string env vars before validation
  const coercedEnv = { ...env };
  
  if (coercedEnv["PORT"] !== undefined) {
    coercedEnv["PORT"] = Number(coercedEnv["PORT"]) as any;
  }
  if (coercedEnv["SESSION_TTL_DAYS"] !== undefined) {
    coercedEnv["SESSION_TTL_DAYS"] = Number(coercedEnv["SESSION_TTL_DAYS"]) as any;
  }
  if (coercedEnv["MAGIC_LINK_TTL_MINUTES"] !== undefined) {
    coercedEnv["MAGIC_LINK_TTL_MINUTES"] = Number(coercedEnv["MAGIC_LINK_TTL_MINUTES"]) as any;
  }
  if (coercedEnv["SMTP_PORT"] !== undefined) {
    coercedEnv["SMTP_PORT"] = Number(coercedEnv["SMTP_PORT"]) as any;
  }
  if (coercedEnv["ENABLE_CONSOLE_EMAIL"] !== undefined) {
    coercedEnv["ENABLE_CONSOLE_EMAIL"] = (coercedEnv["ENABLE_CONSOLE_EMAIL"] === "true") as any;
  }

  // Value.Default applies TypeBox schema defaults to missing optional fields
  const configWithDefaults = Value.Default(ConfigSchema, coercedEnv);

  if (!Value.Check(ConfigSchema, configWithDefaults)) {
    const errors = [...Value.Errors(ConfigSchema, configWithDefaults)];
    const errorDetails = errors.map((e) => `${e.path}: ${e.message}`).join("\n");
    throw new Error(`Configuration Validation Error:\n${errorDetails}`);
  }

  return configWithDefaults as Config;
}

// Global singleton for true composition roots
export const config = loadConfig();
