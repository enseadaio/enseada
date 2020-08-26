import { Method } from "./Method";
import { ResponseError } from "./ResponseError";

export type HttpHeaders = Headers | Record<string, string>;
export type Query = Record<string, string>;
export type AuthTokenProvider = () => Promise<string>;

export interface HttpClientProps {
  baseUrl?: string;
  accessTokenProvider: AuthTokenProvider;
}

export class HttpClient {
  readonly baseUrl?: string;
  private readonly accessTokenProvider: AuthTokenProvider;

  constructor({ baseUrl, accessTokenProvider }: HttpClientProps) {
    this.baseUrl = baseUrl || window.location.origin;
    this.accessTokenProvider = accessTokenProvider;
  }

  get(url: string, query?: Query, headers?: HttpHeaders): Promise<Response> {
    return this.request(Method.GET, url, undefined, query, headers)
  }

  post(url: string, body?: any, query?: Query, headers?: HttpHeaders): Promise<Response> {
    return this.request(Method.POST, url, body, query, headers)
  }

  put(url: string, body?: any, query?: Query, headers?: HttpHeaders): Promise<Response> {
    return this.request(Method.PUT, url, body, query, headers)
  }

  delete(url: string, query?: Query, headers?: HttpHeaders): Promise<Response> {
    return this.request(Method.DELETE, url, undefined, query, headers)
  }

  async request(method: Method, url: string, body?: any, query?: Query, headers?: HttpHeaders): Promise<Response> {
    const h = await this.buildHeaders(headers);
    const uri = this.buildUrl(url, query);
    const res = await fetch(uri.toString(), {
      method,
      mode: 'cors',
      headers: h,
      body: JSON.stringify(body),
    });

    if (res.status >= 400) {
      throw new ResponseError(res);
    }
    return res;
  }

  private buildUrl(url: string, query?: Query): URL {
    let uri: URL;
    if (this.baseUrl && url.startsWith('/')) {
      uri = new URL(url, this.baseUrl);
    } else {
      uri = new URL(url);
    }

    if (query) {
      Object.entries(query).forEach(([key, value]) => {
        uri.searchParams.set(key, value);
      });
    }

    return uri;
  }

  private async buildHeaders(h?: HttpHeaders): Promise<HttpHeaders> {
    const token = await this.accessTokenProvider();
    const tokenValue = `Bearer ${token}`;
    const headers = h || {};
    if (headers instanceof Headers) {
      headers.append('Authorization', tokenValue);
      headers.append('Content-Type', 'application/json');
    } else {
      headers['Authorization'] = tokenValue;
      headers['Content-Type'] = 'application/json';
    }

    return headers;
  }
}