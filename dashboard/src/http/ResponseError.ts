export class ResponseError extends Error {
  readonly status: number;
  readonly response: Response;
  constructor(res: Response, message?: string) {
    super(message || res.statusText);

    this.status = res.status;
    this.response = res;
  }

  get isClient() {
    return this.status >= 400 && this.status < 500;
  }

  get isServer() {
    return this.status >= 500;
  }
}