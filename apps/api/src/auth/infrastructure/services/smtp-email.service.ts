import { IEmailService } from '../../domain/ports/IEmailService';
import * as nodemailer from 'nodemailer';

export class SmtpEmailService implements IEmailService {
  #transporter: nodemailer.Transporter;
  readonly #fromEmail: string;

  constructor() {
    this.#transporter = nodemailer.createTransport({
      host: process.env['SMTP_HOST'],
      port: Number(process.env['SMTP_PORT']) || 587,
      auth: {
        user: process.env['SMTP_USER'],
        pass: process.env['SMTP_PASS'],
      },
    });
    this.#fromEmail = process.env['EMAIL_FROM'] || 'noreply@haven.app';
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
