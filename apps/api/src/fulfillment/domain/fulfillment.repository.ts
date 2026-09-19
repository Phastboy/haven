import { Fulfillment, FulfillmentStatus } from './fulfillment.schema';

export interface IFulfillmentRepository {
  createFulfillment(data: {
    id: string;
    orderId: string;
  }): Promise<Fulfillment>;

  getFulfillmentById(id: string): Promise<Fulfillment | null>;
  
  getFulfillmentByOrderId(orderId: string): Promise<Fulfillment | null>;

  updateFulfillmentStatus(
    id: string,
    status: FulfillmentStatus,
    options?: { deliveryMessage?: string; reviewDeadline?: Date }
  ): Promise<Fulfillment>;

  getExpiredFulfillments(currentDate: Date): Promise<Fulfillment[]>;
}
