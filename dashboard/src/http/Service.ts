import { HttpClient, Query } from "./HttpClient";
import { accessTokenProvider } from '../store';
import { Page, PageParams } from "./Page";

export type Associations = Record<string, Service<Tre>>
export interface Service<T> {
  list(params: PageParams): Promise<Page<T>>;
  get(id: string): Promise<T | undefined>;
  create(payload: any): Promise<T>;
  delete(id: string): Promise<void>;
}

class ServiceImpl<T> implements Service<T>{
  constructor(private readonly path: string, private readonly http: HttpClient) {}

  list(params: PageParams = { limit: 25, offset: 0}): Promise<Page<T>> {
    return this.http.get(this.path, params as Query).then((res) => res.json());
  }

  async get(id: string): Promise<T | undefined> {
    try {
      const res = await this.http.get(`${this.path}/${id}`);
      return res.json();
    } catch (e) {
      if (!e.response || e.response.status != 404) {
        throw e
      }

      return undefined;
    }
  }

  create(payload: any): Promise<T> {
    return this.http.post(this.path, payload).then((res) => res.json());
  }

  async delete(id: string): Promise<void> {
    await this.http.delete(`${this.path}/${id}`);
  }
}

const http = new HttpClient({
  accessTokenProvider,
});

export function createService<T>(path: string): Service<T> {
  return new ServiceImpl<T>(path, http);
}