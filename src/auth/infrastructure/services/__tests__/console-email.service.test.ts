import { expect, test, describe, spyOn } from 'bun:test';
import { ConsoleEmailService } from '../console-email.service';

describe('ConsoleEmailService', () => {
  test('send logs email details to stdout', async () => {
    const consoleSpy = spyOn(console, 'log');
    process.env['ENABLE_CONSOLE_EMAIL'] = 'true';
    const service = new ConsoleEmailService();

    await service.send('test@example.com', 'Test Subject', 'Test Body');

    expect(consoleSpy).toHaveBeenCalled();
    const logOutput = consoleSpy.mock.calls.flat().join(' ');
    expect(logOutput).toContain('test@example.com');
    expect(logOutput).toContain('Test Subject');
    expect(logOutput).toContain('Test Body');

    consoleSpy.mockRestore();
  });
});
