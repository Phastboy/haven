export abstract class DomainError extends Error {
  public readonly type: string;
  public readonly title: string;
  public readonly status: number;
  public readonly detail: string;

  constructor(type: string, title: string, status: number, detail: string) {
    super(detail);
    this.name = this.constructor.name;
    this.type = type;
    this.title = title;
    this.status = status;
    this.detail = detail;
    Error.captureStackTrace(this, this.constructor);
  }
}
