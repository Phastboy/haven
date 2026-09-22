export class ThreadNotFoundError extends Error {
  constructor() {
    super("Thread not found.");
    this.name = "ThreadNotFoundError";
  }
}

export class UnauthorizedThreadAccessError extends Error {
  constructor() {
    super("You do not have permission to access this thread.");
    this.name = "UnauthorizedThreadAccessError";
  }
}

export class MessageNotFoundError extends Error {
  constructor() {
    super("Message not found.");
    this.name = "MessageNotFoundError";
  }
}

export class ParticipantNotFoundError extends Error {
  constructor() {
    super("Participant not found.");
    this.name = "ParticipantNotFoundError";
  }
}

export class SelfThreadError extends Error {
  constructor() {
    super("Cannot create a thread with yourself.");
    this.name = "SelfThreadError";
  }
}
