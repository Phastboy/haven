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
