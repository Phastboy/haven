import 'dotenv/config';
import { SQL } from 'bun';

if (!process.env['DATABASE_URL']) {
  throw new Error('DATABASE_URL environment variable is missing.');
}

export const db = new SQL(process.env['DATABASE_URL']);
