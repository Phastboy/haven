export class OrderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "OrderError";
  }
}

export class OrderNotFoundError extends OrderError {
  constructor() {
    super("Order not found.");
    this.name = "OrderNotFoundError";
  }
}

export class UnauthorizedOrderActionError extends OrderError {
  constructor() {
    super("You are not authorized to perform this action on this order.");
    this.name = "UnauthorizedOrderActionError";
  }
}

export class SelfOrderNotAllowedError extends OrderError {
  constructor() {
    super("You cannot order your own offer.");
    this.name = "SelfOrderNotAllowedError";
  }
}

export class InvalidOrderStateTransitionError extends OrderError {
  constructor(message: string) {
    super(message);
    this.name = "InvalidOrderStateTransitionError";
  }
}

export class OfferNotFoundError extends OrderError {
  constructor() {
    super("Offer not found.");
    this.name = "OfferNotFoundError";
  }
}

export class OfferNotActiveError extends OrderError {
  constructor() {
    super("The requested offer is no longer active.");
    this.name = "OfferNotActiveError";
  }
}

export class DuplicateOrderError extends OrderError {
  constructor() {
    super("You already have a pending order for this offer.");
    this.name = "DuplicateOrderError";
  }
}
