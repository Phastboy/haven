/** Converts a Date to ISO 8601 string. Drizzle returns Date objects for timestamps. */
export function toIso(date: Date): string {
  return date.toISOString();
}

export function toIsoOrNull(date: Date | null): string | null {
  return date === null ? null : date.toISOString();
}

export function toIsoOrUndefined(date: Date | null | undefined): string | undefined {
  return date == null ? undefined : date.toISOString();
}
