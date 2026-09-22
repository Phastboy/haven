import { randomUUID } from "crypto";
import { IOrderRepository } from "../domain/order.repository";
import { Order } from "../domain/order.schema";
import {
  SelfOrderNotAllowedError,
  OfferNotActiveError,
  OfferNotFoundError,
  DuplicateOrderError,
} from "../domain/errors";

export interface IOfferService {
  getOfferPriceAndOwnerAndStatus(
    offerId: string,
  ): Promise<{ price: number; ownerId: string; status: string } | null>;
}

export class CreateOrderUseCase {
  constructor(
    private readonly orderRepository: IOrderRepository,
    private readonly offerService: IOfferService,
  ) {}

  async execute(params: {
    offerId: string;
    requesterId: string;
    quantity: number;
    message?: string;
  }): Promise<Order> {
    const offer = await this.offerService.getOfferPriceAndOwnerAndStatus(params.offerId);

    if (!offer) {
      throw new OfferNotFoundError();
    }

    if (offer.status !== "ACTIVE") {
      throw new OfferNotActiveError();
    }

    if (offer.ownerId === params.requesterId) {
      throw new SelfOrderNotAllowedError();
    }

    const existing = await this.orderRepository.findPendingByRequesterAndOffer(
      params.requesterId,
      params.offerId,
    );
    if (existing) {
      throw new DuplicateOrderError();
    }

    const orderId = randomUUID();
    return this.orderRepository.createOrder({
      id: orderId,
      offerId: params.offerId,
      requesterId: params.requesterId,
      price: offer.price,
      quantity: params.quantity,
      ...(params.message && { message: params.message }),
    });
  }
}
