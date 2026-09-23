import { DomainError } from "../../shared/domain/errors";

export class ThreadNotFoundError extends DomainError {
  constructor() {
    super("thread_not_found", "Thread Not Found", 404, "Thread not found.");
  }
}

export class UnauthorizedThreadAccessError extends DomainError {
  constructor() {
    super(
      "unauthorized_thread_access",
      "Unauthorized Access",
      403,
      "You do not have permission to access this thread.",
    );
  }
}

export class MessageNotFoundError extends DomainError {
  constructor() {
    super("message_not_found", "Message Not Found", 404, "Message not found.");
  }
}

export class ParticipantNotFoundError extends DomainError {
  constructor() {
    super("participant_not_found", "Participant Not Found", 404, "Participant not found.");
  }
}

export class SelfThreadError extends DomainError {
  constructor() {
    super(
      "self_thread",
      "Invalid Thread Participant",
      400,
      "Cannot create a thread with yourself.",
    );
  }
}
