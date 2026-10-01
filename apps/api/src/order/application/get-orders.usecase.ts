import type { IOrderRepository } from "../domain/order.repository";
import type { Order } from "../domain/order.schema";
import { createPaginatedResponse, type PaginatedResponse } from "../../shared/domain/pagination";

export class GetOrdersUseCase {
  constructor(private readonly orderRepository: IOrderRepository) {}

  async getRequesterOrders(requesterId: string, limit = 50, offset = 0): Promise<PaginatedResponse<Order>> {
    const data = await this.orderRepository.getOrdersByRequester(requesterId, limit, offset);
    return createPaginatedResponse(data, { total: data.length });
  }

  async getReceivedOrders(ownerId: string, limit = 50, offset = 0): Promise<PaginatedResponse<Order>> {
    const data = await this.orderRepository.getOrdersByOfferOwner(ownerId, limit, offset);
    return createPaginatedResponse(data, { total: data.length });
  }
}
