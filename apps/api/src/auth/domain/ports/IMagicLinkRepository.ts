import type { MagicLink } from "../magic-link.schema";

export interface CreateMagicLinkDTO {
  email: string;
  token: string;
  expiresAt: string;
}

export interface IMagicLinkRepository {
  create(data: CreateMagicLinkDTO): Promise<MagicLink>;
  findByToken(token: string): Promise<MagicLink | null>;
  markUsed(id: string, usedAt: string): Promise<boolean>;
}
