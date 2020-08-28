import { HttpClient, Query } from "./HttpClient";
import { accessTokenProvider } from '../store';
import { Page, PageParams } from "./Page";

export interface Service<T> {
  readonly http: HttpClient;

  list(params: PageParams = { limit: 25, offset: 0 }): Promise<Page<T>>;

  get(id: string): Promise<T | undefined>;

  exists(id: string): Promise<true>;

  create(payload: any): Promise<T>;

  touch(id: string, payload?: any): Promise<T>;

  update(id: string, payload: any): Promise<T>;

  remove(id?: string, payload?: any): Promise<void>;

  association<A>(type: string, id: string): Service<A>;
}

type AssociationsMap = { [type: string]: (id: string) => Service<any> }

class ServiceImpl<T> implements Service<T> {
  constructor(
    private readonly path: string,
    readonly http: HttpClient,
    private readonly associations: AssociationsMap = {}) {
  }

  list(params: PageParams = { limit: 25, offset: 0 }): Promise<Page<T>> {
    return this.http.get(this.path, params as Query).then((res) => res.json());
  }

  async get(id: string): Promise<T | undefined> {
    const res = await this.http.get(`${this.path}/${id}`);
    return res.json();
  }

  async exists(id: string): Promise<true> {
    await this.http.head(`${this.path}/${id}`);
    return true;
  }

  create(payload: any): Promise<T> {
    return this.http.post(this.path, payload).then((res) => res.json());
  }

  touch(id: string, payload?: any): Promise<T> {
    return this.http.put(`${this.path}/${id}`, payload).then((res) => res.json())
  }

  update(id: string, payload: any): Promise<T> {
    return this.http.put(`${this.path}/${id}`, payload).then((res) => res.json())
  }

  async remove(id?: string, payload?: any): Promise<void> {
    const path = !!id ? `${this.path}/${id}` : this.path;
    await this.http.delete(path, payload);
  }

  association<A>(type: string, id: string): Service<A> {
    const factory = this.associations[type];
    if (!factory) {
      throw new Error(`API ${this.path} has no association of type ${type}`)
    }
    return factory(id);
  }
}

const http = new HttpClient({
  accessTokenProvider,
});

export function createService<T>(path: string, associations: AssociationsMap = {}): Service<T> {
  return new ServiceImpl<T>(path, http, associations);
}