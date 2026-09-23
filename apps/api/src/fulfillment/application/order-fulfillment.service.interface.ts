export interface IOrderFulfillmentService {
  getOrderDetails(orderId: string): Promise<{
    id: string;
    requesterId: string;
    offerId: string;
    status: string;
  } | null>;

  getOfferTypeAndOwner(offerId: string): Promise<{
    offerType: string;
    ownerId: string;
  } | null>;

  updateOrderStatus(orderId: string, status: string): Promise<void>;
}
