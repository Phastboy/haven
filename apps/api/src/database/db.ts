import postgres from "postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import * as schema from "./schema";
import type { Config } from "../config";

/**
 * Factory to create a database connection.
 * @param cfg The config containing the database URL
 */
export function createDb(cfg: Config) {
  const client = postgres(cfg.DATABASE_URL);
  return drizzle(client, { schema });
}

export type DB = ReturnType<typeof createDb>;
