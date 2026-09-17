import { IMagicLinkRepository } from "../../domain/ports/IMagicLinkRepository";
import { IEmailService } from "../../domain/ports/IEmailService";
import { TokenService } from "../../infrastructure/services/token.service";

export class RequestMagicLinkUseCase {
  constructor(
    private magicLinkRepo: IMagicLinkRepository,
    private emailService: IEmailService,
    private tokenService: TokenService,
  ) {}

  async execute(email: string): Promise<void> {
    const ttlMinutes = Number(process.env["MAGIC_LINK_TTL_MINUTES"]) || 15;
    const expiresAt = new Date(Date.now() + ttlMinutes * 60 * 1000).toISOString();

    // Generate raw token to email to the user
    const rawToken = this.tokenService.generate(32);
    // Hash it for DB storage
    const hashedToken = this.tokenService.hash(rawToken);

    await this.magicLinkRepo.create({
      email,
      token: hashedToken,
      expiresAt,
    });

    // We make sure account exists just for tracking or link platform purposes?
    // Wait, magic link usually creates account if it doesn't exist upon verification.
    // So we don't create account here.

    const baseUrl = process.env["MAGIC_LINK_BASE_URL"];
    if(!baseUrl) throw new Error("MAGIC_LINK_BASE_URL not set");

    // Include email and rawToken in the link so frontend can pass it to backend
    const link = `${baseUrl}/auth/magic-login?token=${rawToken}&email=${encodeURIComponent(email)}`;

    const body = `Hello,\n\nClick the link below to login:\n${link}\n\nThis link will expire in ${ttlMinutes} minutes.`;

    await this.emailService.send(email, "Your Magic Login Link", body);
  }
}
