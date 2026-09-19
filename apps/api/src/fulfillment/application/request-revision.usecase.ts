import { IFulfillmentRepository } from '../domain/fulfillment.repository';
import { IOrderFulfillmentService } from './order-fulfillment.service.interface';
import { Fulfillment } from '../domain/fulfillment.schema';
import { FulfillmentNotFoundError, UnauthorizedFulfillmentActionError, InvalidFulfillmentStateTransitionError, RevisionNotApplicableError } from '../domain/errors';

export class RequestRevisionUseCase {
  constructor(
    private readonly fulfillmentRepo: IFulfillmentRepository,
    private readonly orderFulfillmentService: IOrderFulfillmentService
  ) {}

  async execute(params: { orderId: string; accountId: string; reason: string }): Promise<Fulfillment> {
    const order = await this.orderFulfillmentService.getOrderDetails(params.orderId);
    if (!order) {
      throw new Error('Order not found.');
    }

    if (order.requesterId !== params.accountId) {
      throw new UnauthorizedFulfillmentActionError('Only the requester can request a revision.');
    }

    const offer = await this.orderFulfillmentService.getOfferTypeAndOwner(order.offerId);
    if (!offer) {
      throw new Error('Offer not found.');
    }

    if (offer.offerType !== 'SERVICE') {
      throw new RevisionNotApplicableError();
    }

    const fulfillment = await this.fulfillmentRepo.getFulfillmentByOrderId(params.orderId);
    if (!fulfillment) {
      throw new FulfillmentNotFoundError();
    }

    if (fulfillment.status !== 'DELIVERED') {
      throw new InvalidFulfillmentStateTransitionError(`Cannot request revision from status: ${fulfillment.status}`);
    }

    // A revision request removes the deadline until owner delivers again
    return this.fulfillmentRepo.updateFulfillmentStatus(fulfillment.id, 'REVISION_REQUESTED', {
      deliveryMessage: `Revision Requested: ${params.reason}`,
      reviewDeadline: null as any // Hack for Drizzle typing, or undefined if the repo handles null. Let's make repo handle null.
    });
  }
}
