import { DomainError } from "../../shared/domain/errors";

export class OfferError extends DomainError {
  constructor(
    message: string,
    status: number = 400,
    type: string = "offer_error",
    title: string = "Offer Error",
  ) {
    super(type, title, status, message);
  }
}

export class OfferNotFoundError extends OfferError {
  constructor() {
    super("Offer not found.", 404, "offer_not_found", "Offer Not Found");
  }
}

export class UnauthorizedOfferActionError extends OfferError {
  constructor() {
    super(
      "You do not have permission to modify this offer",
      403,
      "unauthorized_offer_action",
      "Unauthorized Action",
    );
  }
}
