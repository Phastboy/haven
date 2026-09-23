import type { IOrderRepository } from "../domain/order.repository";
import type { Order } from "../domain/order.schema";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class GetOrdersUseCase {
  constructor(private readonly orderRepository: IOrderRepository) {}

  async getRequesterOrders(requesterId: string): Promise<PaginatedResponse<Order>> {
    const data = await this.orderRepository.getOrdersByRequester(requesterId);
    return createPaginatedResponse(data, { total: data.length });
  }

  async getReceivedOrders(ownerId: string): Promise<PaginatedResponse<Order>> {
    const data = await this.orderRepository.getOrdersByOfferOwner(ownerId);
    return createPaginatedResponse(data, { total: data.length });
  }
}
