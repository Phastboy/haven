import 'dotenv/config';
import { db } from './db';
import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';

async function setup() {
  try {
    console.log('Ensuring migrations table exists...');
    await db.unsafe(`
      CREATE TABLE IF NOT EXISTS "_Migrations" (
        "id" SERIAL PRIMARY KEY,
        "name" TEXT UNIQUE NOT NULL,
        "executedAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
    `);

    const migrationsDir = join(import.meta.dir, '../../migrations');
    const files = readdirSync(migrationsDir)
      .filter((file) => file.endsWith('.sql'))
      .sort();

    const executedRows = await db`SELECT name FROM "_Migrations"`;
    const executedMigrations = new Set(executedRows.map((r) => r.name));

    for (const file of files) {
      if (!executedMigrations.has(file)) {
        console.log(`Running migration: ${file}`);
        const sql = readFileSync(join(migrationsDir, file), 'utf-8');
        
        const tx = db.begin();
        try {
          await tx.unsafe(sql);
          await tx`INSERT INTO "_Migrations" (name) VALUES (${file})`;
          await tx.commit();
          console.log(`Migration ${file} applied successfully.`);
        } catch (err) {
          await tx.rollback();
          console.error(`Failed to apply migration ${file}:`, err);
          throw err;
        }
      }
    }

    console.log('Database migrations successfully applied.');
  } catch (error) {
    console.error('Error setting up the database:', error);
    process.exit(1);
  } finally {
    db.close();
  }
}

setup();
