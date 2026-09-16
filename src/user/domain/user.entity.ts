/**
 * The User entity as the domain understands it.
 * Deliberately independent of the ORM — the domain must not import from infrastructure.
 */
export interface User {
  id: string;
  email: string;
  username: string | null;
  name: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateUserData {
  email: string;
  username?: string | null;
  name?: string | null;
}

export interface UpdateUserData {
  username?: string | null;
  name?: string | null;
}
