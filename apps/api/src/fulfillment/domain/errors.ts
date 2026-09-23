import { DomainError } from "../../shared/domain/errors";

export class FulfillmentError extends DomainError {
  constructor(
    message: string,
    status: number = 400,
    type: string = "fulfillment_error",
    title: string = "Fulfillment Error",
  ) {
    super(type, title, status, message);
  }
}

export class FulfillmentNotFoundError extends FulfillmentError {
  constructor() {
    super("Fulfillment not found.", 404, "fulfillment_not_found", "Fulfillment Not Found");
  }
}

export class OrderNotFoundForFulfillmentError extends FulfillmentError {
  constructor() {
    super("Order not found.", 404, "order_not_found", "Order Not Found");
  }
}

export class UnauthorizedFulfillmentActionError extends FulfillmentError {
  constructor(
    message: string = "You are not authorized to perform this action on this fulfillment.",
  ) {
    super(message, 403, "unauthorized_fulfillment_action", "Unauthorized Action");
  }
}

export class InvalidFulfillmentStateTransitionError extends FulfillmentError {
  constructor(message: string) {
    super(message, 422, "invalid_fulfillment_state_transition", "Invalid State Transition");
  }
}

export class RevisionNotApplicableError extends FulfillmentError {
  constructor() {
    super(
      "Revisions can only be requested for SERVICE offers.",
      400,
      "revision_not_applicable",
      "Revision Not Applicable",
    );
  }
}
