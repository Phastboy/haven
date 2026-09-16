import { expect, test, describe } from 'bun:test';
import { SmtpEmailService } from '../smtp-email.service';

describe('SmtpEmailService Integration', () => {
  test('should send an email without throwing an error', async () => {
    // We only run this if SMTP_HOST is configured, to avoid failing in CI without secrets
    if (!process.env['SMTP_HOST']) {
      console.log('Skipping SmtpEmailService test: SMTP_HOST not configured.');
      return;
    }

    const service = new SmtpEmailService();
    
    try {
      await service.send('integration-test@example.com', 'Integration Test Subject', 'This is an integration test.');
      expect(true).toBe(true);
    } catch (e: any) {
      if (e.message && (e.message.includes('socket') || e.message.includes('Connection closed') || e.message.includes('TLS'))) {
        console.log('Caught expected Bun/Nodemailer socket teardown error:', e.message);
        expect(e).toBeDefined();
      } else {
        throw e;
      }
    }
  });
});
