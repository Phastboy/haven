export class OfferError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'OfferError';
  }
}

export class OfferNotFoundError extends OfferError {
  constructor(id: string) {
    super(`Offer with ID ${id} not found`);
    this.name = 'OfferNotFoundError';
  }
}

export class UnauthorizedOfferActionError extends OfferError {
  constructor() {
    super('You do not have permission to modify this offer');
    this.name = 'UnauthorizedOfferActionError';
  }
}
