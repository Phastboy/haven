import { DomainError } from "../../shared/domain/errors";

export class OrderError extends DomainError {
  constructor(
    message: string,
    status: number = 400,
    type: string = "order_error",
    title: string = "Order Error",
  ) {
    super(type, title, status, message);
  }
}

export class OrderNotFoundError extends OrderError {
  constructor() {
    super("Order not found.", 404, "order_not_found", "Order Not Found");
  }
}

export class UnauthorizedOrderActionError extends OrderError {
  constructor() {
    super(
      "You are not authorized to perform this action on this order.",
      403,
      "unauthorized_order_action",
      "Unauthorized Action",
    );
  }
}

export class SelfOrderNotAllowedError extends OrderError {
  constructor() {
    super(
      "You cannot order your own offer.",
      400,
      "self_order_not_allowed",
      "Invalid Order Target",
    );
  }
}

export class InvalidOrderStateTransitionError extends OrderError {
  constructor(message: string) {
    super(message, 422, "invalid_order_state_transition", "Invalid State Transition");
  }
}

export class OfferNotFoundError extends OrderError {
  constructor() {
    super("Offer not found.", 404, "offer_not_found", "Offer Not Found");
  }
}

export class OfferNotActiveError extends OrderError {
  constructor() {
    super("The requested offer is no longer active.", 400, "offer_not_active", "Offer Not Active");
  }
}

export class DuplicateOrderError extends OrderError {
  constructor() {
    super(
      "You already have a pending order for this offer.",
      409,
      "duplicate_order",
      "Duplicate Order",
    );
  }
}
