
import { DomainError } from "../domain/errors";

const getHttpStatusPhrase = (status: number): string => {
  switch (status) {
    case 400:
      return "Bad Request";
    case 401:
      return "Unauthorized";
    case 403:
      return "Forbidden";
    case 404:
      return "Not Found";
    case 409:
      return "Conflict";
    case 422:
      return "Unprocessable Entity";
    default:
      return "Internal Server Error";
  }
};

/**
 * Global last-resort error handler Plugin.
 * Ensures no raw SQL, stack traces, or Postgres internals ever reach the client
 * for errors that no plugin handler claimed.
 */
export const globalErrorHandler = (context: any): any => {
    const { error, set, request } = context;
    const asRecordContext = context as unknown as Record<string, unknown>;
    const asRecordError = error as unknown as Record<string, unknown>;
    const code = asRecordContext["code"] || asRecordError["code"];
    const instance = new URL(request.url).pathname;

    // 1. Custom Domain Errors
    if (error instanceof DomainError) {
      set.status = error.status;
      return {
        type: "about:blank",
        title: getHttpStatusPhrase(error.status),
        status: error.status,
        detail: error.detail,
        instance,
      };
    }

    // 2. Validation Errors (TypeBox/Elysia)
    if (error && error.constructor.name === "ValidationError") {
      set.status = 422;

      let validationErrors: { name: string; reason: string }[] | undefined = undefined;

      const extractErrors = (errors: unknown[]) => {
        const mapped = errors.map((err) => {
          const asRecord = err as Record<string, unknown>;
          let name = String(asRecord["instancePath"] || asRecord["path"] || "");
          if (name === "" || name === "root") {
            const schemaPath = String(asRecord["schemaPath"] || "");
            const match = schemaPath.match(/#\/properties\/([^\/]+)/);
            name = match?.[1] || "body";
          } else {
            name = name.replace(/^\//, "");
          }
          return { name, reason: String(asRecord["message"] || "") };
        });

        const errorMap = new Map<string, string[]>();
        for (const e of mapped) {
          if (!errorMap.has(e.name)) {
            errorMap.set(e.name, []);
          }
          if (e.reason !== "must match a schema in anyOf" && e.reason !== "Expected union value") {
            errorMap.get(e.name)!.push(e.reason);
          }
        }

        return Array.from(errorMap.entries()).map(([name, reasons]) => ({
          name,
          reason: reasons.length > 0 ? reasons.join(" OR ") : "Invalid value",
        }));
      };

      const asRecordError = error as Record<string, unknown>;
      if (asRecordError["errors"] && typeof asRecordError["errors"] === "object") {
        validationErrors = extractErrors(Array.from(asRecordError["errors"] as Iterable<unknown>));
      }

      return {
        type: "about:blank",
        title: "Unprocessable Entity",
        status: 422,
        detail: "The request payload failed to validate against the schema.",
        instance,
        errors: validationErrors,
      };
    }

    // 3. Parse Errors (Malformed JSON)
    if (code === "PARSE") {
      set.status = 400;
      set.headers = Object.assign(set.headers || {}, { "content-type": "application/problem+json" });
      return {
        type: "about:blank",
        title: "Bad Request",
        status: 400,
        detail: "The request payload is malformed or invalid JSON.",
        instance,
      };
    }

    // 4. Not Found Errors (Elysia unmatched routes)
    const asAny = error as unknown as Record<string, unknown>;
    if (
      code === "NOT_FOUND" ||
      asAny["code"] === "not-found" ||
      (error instanceof Error &&
        "status" in error &&
        (error as unknown as { status: number }).status === 404)
    ) {
      set.status = 404;
      return {
        type: "about:blank",
        title: "Not Found",
        status: 404,
        detail: "The requested route or resource does not exist.",
        instance,
      };
    }

    // 4. Unhandled Internal Errors
    console.error("[unhandled error]", error);
    set.status = 500;
    return {
      type: "about:blank",
      title: "Internal Server Error",
      status: 500,
      detail: "An unexpected error occurred.",
      instance,
    };
  };
