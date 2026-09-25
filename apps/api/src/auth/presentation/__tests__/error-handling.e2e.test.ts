import { expect, test, describe, spyOn } from "bun:test";
import { app } from "../../../index";
import { RequestMagicLinkUseCase } from "../../application/use-cases/request-magic-link.use-case";

interface ErrorResponse {
  type: string;
  title: string;
  status: number;
  detail: string;
  instance: string;
  errors?: { name: string; reason: string }[];
}

describe("Global Error Handling E2E", () => {
  test("should return 422 Unprocessable Entity for validation failures, conforming to RFC 9457", async () => {
    // Send a payload that violates the schema (missing email)
    const request = new Request("http://localhost/api/auth/magic-link/request", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: 12345 }), // Invalid email type
    });

    const response = await app.handle(request);
    expect(response.status).toBe(422);

    const body = (await response.json()) as ErrorResponse;

    // Check RFC 9457 base fields
    expect(body.type).toBe("about:blank");
    expect(body.title).toBe("Unprocessable Entity");
    expect(body.status).toBe(422);
    expect(body.detail).toBe("The request payload failed to validate against the schema.");
    expect(body.instance).toBe("/api/auth/magic-link/request");

    // Check specific validation errors
    expect(body.errors).toBeDefined();
    expect(Array.isArray(body.errors)).toBe(true);
    expect(body.errors!.length).toBeGreaterThan(0);
    expect(body.errors![0]!.name).toBeDefined();
    expect(body.errors![0]!.reason).toBeDefined();
  });

  test("should return 500 Internal Server Error for unexpected exceptions without leaking internals", async () => {
    // Spy on the use case and force an unexpected exception
    const originalExecute = RequestMagicLinkUseCase.prototype.execute;

    spyOn(RequestMagicLinkUseCase.prototype, "execute").mockImplementation(async () => {
      throw new Error("CRITICAL_DB_FAILURE_DO_NOT_LEAK_ME");
    });

    const request = new Request("http://localhost/api/auth/magic-link/request", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: "test@example.com" }),
    });

    const response = await app.handle(request);

    // Restore immediately
    RequestMagicLinkUseCase.prototype.execute = originalExecute;

    expect(response.status).toBe(500);
    const body = (await response.json()) as ErrorResponse;

    // Check RFC 9457 base fields
    expect(body.type).toBe("about:blank");
    expect(body.title).toBe("Internal Server Error");
    expect(body.status).toBe(500);

    // Crucially: Assert the internal message did NOT leak
    expect(body.detail).toBe("An unexpected error occurred.");
    expect(body.detail).not.toInclude("CRITICAL_DB_FAILURE");

    // Also assert there's no stack trace dumped into the response body
    const responseString = JSON.stringify(body);
    expect(responseString).not.toInclude("Error:");
    expect(responseString).not.toInclude("at ");

    expect(body.instance).toBe("/api/auth/magic-link/request");
  });
});
