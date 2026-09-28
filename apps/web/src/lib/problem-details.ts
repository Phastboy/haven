export interface ProblemDetails {
  type?: string;
  title?: string;
  status?: number;
  detail?: string;
  instance?: string;
  errors?: Array<{ name: string; reason: string }>;
}

/**
 * Safely extracts human-readable error messages from an RFC 9457 Problem Details object,
 * or a generic Eden Treaty error. Returns an array of strings suitable for toast notifications.
 */
export function parseProblemDetails(error: unknown): string[] {
  if (!error || typeof error !== "object") {
    return ["An unexpected error occurred."];
  }

  // Handle Eden Treaty error wrapper (which exposes `.value`)
  const errVal = "value" in error ? (error as { value: unknown }).value : error;

  if (!errVal || typeof errVal !== "object") {
    return ["An unexpected error occurred."];
  }

  const pd = errVal as ProblemDetails;
  const messages: string[] = [];

  if (pd.errors && Array.isArray(pd.errors) && pd.errors.length > 0) {
    pd.errors.forEach((err) => {
      if (err.name && err.name !== "root" && err.name !== "") {
        messages.push(`${err.name}: ${err.reason}`);
      } else if (err.reason) {
        messages.push(err.reason);
      }
    });
  } else if (pd.detail) {
    messages.push(pd.detail);
  }

  if (messages.length === 0) {
    messages.push(pd.title || "An unexpected error occurred.");
  }

  return messages;
}
