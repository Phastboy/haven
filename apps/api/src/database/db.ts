import postgres from "postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import * as schema from "./schema";
import type { Config } from "../config";

/**
 * Factory to create a database connection.
 * @param cfg The config containing the database URL
 */
export function createDb(cfg: Config) {
  const client = postgres(cfg.DATABASE_URL, {
    max: cfg.DB_POOL_MAX,
    idle_timeout: cfg.DB_IDLE_TIMEOUT,
    connect_timeout: 10,
  });
  return drizzle(client, { schema });
}

export type DB = ReturnType<typeof createDb>;
