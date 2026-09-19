export class FulfillmentError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "FulfillmentError";
  }
}

export class FulfillmentNotFoundError extends FulfillmentError {
  constructor() {
    super("Fulfillment not found.");
    this.name = "FulfillmentNotFoundError";
  }
}

export class UnauthorizedFulfillmentActionError extends FulfillmentError {
  constructor(
    message: string = "You are not authorized to perform this action on this fulfillment.",
  ) {
    super(message);
    this.name = "UnauthorizedFulfillmentActionError";
  }
}

export class InvalidFulfillmentStateTransitionError extends FulfillmentError {
  constructor(message: string) {
    super(message);
    this.name = "InvalidFulfillmentStateTransitionError";
  }
}

export class RevisionNotApplicableError extends FulfillmentError {
  constructor() {
    super("Revisions can only be requested for SERVICE offers.");
    this.name = "RevisionNotApplicableError";
  }
}
