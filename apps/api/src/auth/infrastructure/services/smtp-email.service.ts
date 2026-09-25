import type { IEmailService } from "../../domain/ports/IEmailService";
import * as nodemailer from "nodemailer";

import type { Config } from "../../../config";

export class SmtpEmailService implements IEmailService {
  #transporter: nodemailer.Transporter;
  readonly #fromEmail: string;

  constructor(config: Config) {
    this.#transporter = nodemailer.createTransport({
      host: config.SMTP_HOST,
      port: config.SMTP_PORT,
      auth: {
        user: config.SMTP_USER,
        pass: config.SMTP_PASS,
      },
    });
    this.#fromEmail = config.EMAIL_FROM;
  }

  async send(to: string, subject: string, body: string): Promise<void> {
    await this.#transporter.sendMail({
      from: this.#fromEmail,
      to,
      subject,
      text: body, // we could also support HTML here
    });
  }
}
