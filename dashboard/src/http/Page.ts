export interface Page<T> {
  count: number,
  total: number,
  offset: number,
  limit: number,
  items: T[],
}

export interface PageParams {
  limit: number;
  offset: number;
}

export function pageToOffset(page: number, limit: number): number {
  return (page - 1) * limit;
}