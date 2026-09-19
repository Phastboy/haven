import { Order, OrderStatus } from './order.schema';

export interface IOrderRepository {
  createOrder(data: {
    id: string;
    offerId: string;
    requesterId: string;
    price: number;
    quantity: number;
    message?: string | null;
  }): Promise<Order>;

  getOrderById(id: string): Promise<Order | null>;

  getOrdersByRequester(requesterId: string): Promise<Order[]>;

  getOrdersByOfferOwner(ownerId: string): Promise<Order[]>;

  updateOrderStatus(id: string, status: OrderStatus): Promise<Order>;
}
