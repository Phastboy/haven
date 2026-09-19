import { IOrderRepository } from "../domain/order.repository";
import { Order, OrderStatus } from "../domain/order.schema";
import {
  OrderNotFoundError,
  UnauthorizedOrderActionError,
  InvalidOrderStateTransitionError,
} from "../domain/errors";

export interface IOfferOwnerService {
  getOfferOwnerId(offerId: string): Promise<string | null>;
}

export class UpdateOrderStatusUseCase {
  constructor(
    private readonly orderRepository: IOrderRepository,
    private readonly offerOwnerService: IOfferOwnerService,
  ) {}

  async execute(params: {
    orderId: string;
    accountId: string;
    newStatus: OrderStatus;
  }): Promise<Order> {
    const order = await this.orderRepository.getOrderById(params.orderId);
    if (!order) {
      throw new OrderNotFoundError();
    }

    const offerOwnerId = await this.offerOwnerService.getOfferOwnerId(order.offerId);
    if (!offerOwnerId) {
      throw new Error("Offer associated with order not found.");
    }

    const isRequester = order.requesterId === params.accountId;
    const isOwner = offerOwnerId === params.accountId;

    if (!isRequester && !isOwner) {
      throw new UnauthorizedOrderActionError();
    }

    // State machine logic
    if (params.newStatus === "CANCELLED") {
      if (!isRequester) {
        throw new InvalidOrderStateTransitionError("Only the requester can cancel an order.");
      }
      if (order.status !== "PENDING") {
        throw new InvalidOrderStateTransitionError("Only PENDING orders can be cancelled.");
      }
    } else if (params.newStatus === "ACCEPTED" || params.newStatus === "REJECTED") {
      if (!isOwner) {
        throw new InvalidOrderStateTransitionError(
          `Only the offer owner can mark an order as ${params.newStatus}.`,
        );
      }
      if (order.status !== "PENDING") {
        throw new InvalidOrderStateTransitionError(
          `Cannot transition from ${order.status} to ${params.newStatus}.`,
        );
      }
    } else if (params.newStatus === "COMPLETED") {
      if (!isOwner) {
        throw new InvalidOrderStateTransitionError(
          "Only the offer owner can mark an order as COMPLETED.",
        );
      }
      if (order.status !== "ACCEPTED") {
        throw new InvalidOrderStateTransitionError(
          "Only ACCEPTED orders can be marked as COMPLETED.",
        );
      }
    } else if (params.newStatus === "PENDING") {
      throw new InvalidOrderStateTransitionError("Cannot manually revert order back to PENDING.");
    }

    return this.orderRepository.updateOrderStatus(order.id, params.newStatus);
  }
}
