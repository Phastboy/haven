import { IFulfillmentRepository } from '../domain/fulfillment.repository';
import { IOrderFulfillmentService } from './order-fulfillment.service.interface';

export class AutoCompleteExpiredUseCase {
  constructor(
    private readonly fulfillmentRepo: IFulfillmentRepository,
    private readonly orderFulfillmentService: IOrderFulfillmentService
  ) {}

  async execute(): Promise<number> {
    const now = new Date();
    const expiredFulfillments = await this.fulfillmentRepo.getExpiredFulfillments(now);

    let completedCount = 0;

    for (const fulfillment of expiredFulfillments) {
      try {
        await this.fulfillmentRepo.updateFulfillmentStatus(fulfillment.id, 'COMPLETED');
        await this.orderFulfillmentService.updateOrderStatus(fulfillment.orderId, 'COMPLETED');
        completedCount++;
      } catch (e) {
        console.error(`Failed to auto-complete fulfillment ${fulfillment.id}:`, e);
      }
    }

    return completedCount;
  }
}
