import { expect, test, describe, mock, beforeEach } from "bun:test";

const sendMailMock = mock();
const createTransportMock = mock(() => ({
  sendMail: sendMailMock,
}));

mock.module("nodemailer", () => {
  return {
    createTransport: createTransportMock,
  };
});

import { SmtpEmailService } from "../smtp-email.service";

describe("SmtpEmailService", () => {
  beforeEach(() => {
    sendMailMock.mockClear();
    createTransportMock.mockClear();
  });

  test("should initialize transporter with environment variables", () => {
    process.env["SMTP_HOST"] = "smtp.test.com";
    process.env["SMTP_PORT"] = "465";
    process.env["SMTP_USER"] = "testuser";
    process.env["SMTP_PASS"] = "testpass";
    process.env["EMAIL_FROM"] = "test@haven.app";

    new SmtpEmailService();

    expect(createTransportMock).toHaveBeenCalledWith({
      host: "smtp.test.com",
      port: 465,
      auth: {
        user: "testuser",
        pass: "testpass",
      },
    });
  });

  test("should send email successfully", async () => {
    sendMailMock.mockResolvedValueOnce(true);
    const service = new SmtpEmailService();

    await service.send("to@example.com", "Subject", "Body");

    expect(sendMailMock).toHaveBeenCalledWith({
      from: "test@haven.app",
      to: "to@example.com",
      subject: "Subject",
      text: "Body",
    });
  });

  test("should throw when email sending fails (timeout/rejection)", async () => {
    sendMailMock.mockRejectedValueOnce(new Error("SMTP connection timeout"));
    const service = new SmtpEmailService();

    expect(service.send("to@example.com", "Subject", "Body")).rejects.toThrow(
      "SMTP connection timeout",
    );
  });
});
