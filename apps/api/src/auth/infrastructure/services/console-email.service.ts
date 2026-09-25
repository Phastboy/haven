import type { IEmailService } from "../../domain/ports/IEmailService";
import type { Config } from "../../../config";

export class ConsoleEmailService implements IEmailService {
  #enabled: boolean;

  constructor(config: Config) {
    this.#enabled = config.ENABLE_CONSOLE_EMAIL;
  }

  async send(to: string, subject: string, body: string): Promise<void> {
    if (!this.#enabled) {
      return;
    }
    console.log(`To: ${to}`);
    console.log(`Subject: ${subject}`);
    console.log(`Body:\n${body}`);
    console.log(`=========================================\n`);
  }
}
