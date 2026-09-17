import 'dotenv/config';
import { db } from './db';
import { readFileSync } from 'fs';
import { join } from 'path';

async function setup() {
  try {
    const schemaSql = readFileSync(join(import.meta.dir, 'schema.sql'), 'utf-8');
    console.log('Running database schema migrations...');
    await db.unsafe(schemaSql);
    console.log('Database schema successfully set up.');
  } catch (error) {
    console.error('Error setting up the database:', error);
    process.exit(1);
  } finally {
    db.close();
  }
}

setup();
