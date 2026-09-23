import { IFulfillmentRepository } from "../domain/fulfillment.repository";
import { IOrderFulfillmentService } from "./order-fulfillment.service.interface";
import { Fulfillment } from "../domain/fulfillment.schema";
import {
  FulfillmentNotFoundError,
  UnauthorizedFulfillmentActionError,
  InvalidFulfillmentStateTransitionError,
} from "../domain/errors";

export class AcceptFulfillmentUseCase {
  constructor(
    private readonly fulfillmentRepo: IFulfillmentRepository,
    private readonly orderFulfillmentService: IOrderFulfillmentService,
  ) {}

  async execute(params: { orderId: string; accountId: string }): Promise<Fulfillment> {
    const order = await this.orderFulfillmentService.getOrderDetails(params.orderId);
    if (!order) {
      throw new Error("Order not found.");
    }

    if (order.requesterId !== params.accountId) {
      throw new UnauthorizedFulfillmentActionError(
        "Only the requester can accept the fulfillment.",
      );
    }

    const fulfillment = await this.fulfillmentRepo.getFulfillmentByOrderId(params.orderId);
    if (!fulfillment) {
      throw new FulfillmentNotFoundError();
    }

    if (fulfillment.status !== "DELIVERED" && fulfillment.status !== "REVISION_REQUESTED") {
      throw new InvalidFulfillmentStateTransitionError(
        `Cannot accept fulfillment from status: ${fulfillment.status}`,
      );
    }

    const updated = await this.fulfillmentRepo.updateFulfillmentStatus(fulfillment.id, "COMPLETED");

    // Cascade state to order
    await this.orderFulfillmentService.updateOrderStatus(order.id, "COMPLETED");

    return updated;
  }
}
