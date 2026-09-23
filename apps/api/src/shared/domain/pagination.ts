export interface PaginationMeta {
  total?: number;
  cursor?: string | null;
  hasMore: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: PaginationMeta;
}

export function createPaginatedResponse<T>(
  data: T[],
  meta: Partial<PaginationMeta> = {},
): PaginatedResponse<T> {
  const metaResult: PaginationMeta = {
    hasMore: meta.hasMore ?? false,
    cursor: meta.cursor ?? null,
  };

  if (meta.total !== undefined) {
    metaResult.total = meta.total;
  }

  return {
    data,
    meta: metaResult,
  };
}
