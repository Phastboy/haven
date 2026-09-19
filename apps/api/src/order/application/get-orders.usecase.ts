import { IOrderRepository } from '../domain/order.repository';
import { Order } from '../domain/order.schema';

export class GetOrdersUseCase {
  constructor(private readonly orderRepository: IOrderRepository) {}

  async getRequesterOrders(requesterId: string): Promise<Order[]> {
    return this.orderRepository.getOrdersByRequester(requesterId);
  }

  async getReceivedOrders(ownerId: string): Promise<Order[]> {
    return this.orderRepository.getOrdersByOfferOwner(ownerId);
  }
}
