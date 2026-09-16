/**
 * User domain types — derived from TypeBox schemas in user.schema.ts.
 *
 * Re-exported here so the rest of the domain (repository interface, use cases)
 * can import from a single stable path without knowing about the schema file.
 */
export type { User, CreateUserData, UpdateUserData } from './user.schema';

