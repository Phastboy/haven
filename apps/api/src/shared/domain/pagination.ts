import { t } from "elysia";

export interface PaginationMeta {
  total?: number;
  cursor?: string | null;
  hasMore: boolean;
}

export const PaginationMetaSchema = t.Object({
  total: t.Optional(t.Number()),
  cursor: t.Optional(t.Union([t.String(), t.Null()])),
  hasMore: t.Boolean(),
});

export interface PaginatedResponse<T> {
  data: T[];
  meta: PaginationMeta;
}

export const PaginatedResponseSchema = (dataSchema: any) => t.Object({
  data: t.Array(dataSchema),
  meta: PaginationMetaSchema,
});

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
