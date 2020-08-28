export { Repository as ContainerRepository } from './containers';
export * from './users';
export * from './roles';

export type IdMapFn<T> = (obj: T) => string;

export interface Page<T> {
  count: number,
  total: number,
  offset: number,
  limit: number,
  items: T[],
}