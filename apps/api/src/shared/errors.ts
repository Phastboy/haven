import { DomainError } from "./domain/errors";

export class NotFoundError extends DomainError {
  constructor(resource: string, id: string) {
    super("not_found", "Not Found", 404, `${resource} with id "${id}" was not found.`);
  }
}

export class ConflictError extends DomainError {
  constructor(message: string) {
    super("conflict", "Conflict", 409, message);
  }
}

export class ValidationError extends DomainError {
  constructor(message: string) {
    super("validation_error", "Validation Failed", 422, message);
  }
}
