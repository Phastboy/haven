import { Session, SessionWithAccount } from '../session.schema';

export interface CreateSessionDTO {
  accountId: string;
  token: string;
  expiresAt: string;
  userAgent?: string;
  ipAddress?: string;
}

export interface ISessionRepository {
  create(data: CreateSessionDTO): Promise<Session>;
  findByToken(token: string): Promise<SessionWithAccount | null>;
  deleteByToken(token: string): Promise<void>;
  deleteExpired(): Promise<void>;
}
