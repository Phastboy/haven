import { IEmailService } from '../../domain/ports/IEmailService';

export class ConsoleEmailService implements IEmailService {
  async send(to: string, subject: string, body: string): Promise<void> {

    console.log(`\n================= EMAIL =================`);
    console.log(`To: ${to}`);
    console.log(`Subject: ${subject}`);
    console.log(`Body:\n${body}`);
    console.log(`=========================================\n`);
  }
}
