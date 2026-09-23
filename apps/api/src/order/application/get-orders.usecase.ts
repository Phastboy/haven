import type { IOrderRepository } from "../domain/order.repository";
import type { Order } from "../domain/order.schema";

export class GetOrdersUseCase {
  constructor(private readonly orderRepository: IOrderRepository) {}

  async getRequesterOrders(requesterId: string): Promise<Order[]> {
    return this.orderRepository.getOrdersByRequester(requesterId);
  }

  async getReceivedOrders(ownerId: string): Promise<Order[]> {
    return this.orderRepository.getOrdersByOfferOwner(ownerId);
  }
}
