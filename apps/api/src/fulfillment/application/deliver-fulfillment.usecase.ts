import { IFulfillmentRepository } from "../domain/fulfillment.repository";
import { IOrderFulfillmentService } from "./order-fulfillment.service.interface";
import { Fulfillment } from "../domain/fulfillment.schema";
import {
  UnauthorizedFulfillmentActionError,
  InvalidFulfillmentStateTransitionError,
} from "../domain/errors";
import { randomUUID } from "crypto";

export class DeliverFulfillmentUseCase {
  constructor(
    private readonly fulfillmentRepo: IFulfillmentRepository,
    private readonly orderFulfillmentService: IOrderFulfillmentService,
  ) {}

  async execute(params: {
    orderId: string;
    accountId: string;
    deliveryMessage?: string;
    autoReviewDays: number;
  }): Promise<Fulfillment> {
    const order = await this.orderFulfillmentService.getOrderDetails(params.orderId);
    if (!order) {
      throw new Error("Order not found.");
    }

    if (order.status !== "ACCEPTED") {
      throw new InvalidFulfillmentStateTransitionError("Only ACCEPTED orders can be delivered.");
    }

    const offer = await this.orderFulfillmentService.getOfferTypeAndOwner(order.offerId);
    if (!offer || offer.ownerId !== params.accountId) {
      throw new UnauthorizedFulfillmentActionError(
        "Only the offer owner can deliver the fulfillment.",
      );
    }

    let fulfillment = await this.fulfillmentRepo.getFulfillmentByOrderId(params.orderId);

    if (!fulfillment) {
      fulfillment = await this.fulfillmentRepo.createFulfillment({
        id: randomUUID(),
        orderId: params.orderId,
      });
    }

    if (fulfillment.status === "COMPLETED") {
      throw new InvalidFulfillmentStateTransitionError("Fulfillment is already COMPLETED.");
    }

    const reviewDeadline = new Date(Date.now() + params.autoReviewDays * 24 * 60 * 60 * 1000);

    return this.fulfillmentRepo.updateFulfillmentStatus(fulfillment.id, "DELIVERED", {
      ...(params.deliveryMessage && { deliveryMessage: params.deliveryMessage }),
      reviewDeadline,
    });
  }
}
