// ABOUTME: Shared TypeScript types for pagination
// ABOUTME: Defines pagination metadata, request params, and response wrappers

export interface PaginationMetadata {
  page: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: PaginationMetadata;
}

export interface PaginationParams {
  page?: number;
  limit?: number;
}

/**
 * Helper to build query string from pagination params
 */
export function buildPaginationQuery(params: PaginationParams): string {
  const query: string[] = [];
  if (params.page !== undefined) {
    query.push(`page=${params.page}`);
  }
  if (params.limit !== undefined) {
    query.push(`limit=${params.limit}`);
  }
  return query.length > 0 ? `?${query.join('&')}` : '';
}

/**
 * Default pagination parameters
 */
export const DEFAULT_PAGINATION: PaginationParams = {
  page: 1,
  limit: 20,
};
