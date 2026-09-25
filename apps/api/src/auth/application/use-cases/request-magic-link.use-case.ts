import type { IMagicLinkRepository } from "../../domain/ports/IMagicLinkRepository";
import type { IEmailService } from "../../domain/ports/IEmailService";
import type { TokenService } from "../../infrastructure/services/token.service";
import type { Config } from "../../../config";

export class RequestMagicLinkUseCase {
  readonly #magicLinkRepo: IMagicLinkRepository;
  readonly #emailService: IEmailService;
  readonly #tokenService: TokenService;
  readonly #config: Config;

  constructor(
    magicLinkRepo: IMagicLinkRepository,
    emailService: IEmailService,
    tokenService: TokenService,
    config: Config,
  ) {
    this.#magicLinkRepo = magicLinkRepo;
    this.#emailService = emailService;
    this.#tokenService = tokenService;
    this.#config = config;
  }

  async execute(email: string): Promise<void> {
    const ttlMinutes = this.#config.MAGIC_LINK_TTL_MINUTES;
    const expiresAt = new Date(Date.now() + ttlMinutes * 60 * 1000).toISOString();

    // Generate raw token to email to the user
    const rawToken = this.#tokenService.generate(32);
    // Hash it for DB storage
    const hashedToken = this.#tokenService.hash(rawToken);

    await this.#magicLinkRepo.create({
      email,
      token: hashedToken,
      expiresAt,
    });

    // We make sure account exists just for tracking or link platform purposes?
    // Wait, magic link usually creates account if it doesn't exist upon verification.
    // So we don't create account here.

    const baseUrl = this.#config.MAGIC_LINK_BASE_URL;

    // Include email and rawToken in the link so frontend can pass it to backend
    const link = `${baseUrl}/auth/magic-login?token=${rawToken}&email=${encodeURIComponent(email)}`;

    const body = `Hello,\n\nClick the link below to login:\n${link}\n\nThis link will expire in ${ttlMinutes} minutes.`;

    await this.#emailService.send(email, "Your Magic Login Link", body);
  }
}
