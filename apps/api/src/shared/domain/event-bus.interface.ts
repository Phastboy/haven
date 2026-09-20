export interface OrderAcceptedEventPayload {
  orderId: string;
  requesterId: string;
  ownerId: string;
}

export interface DomainEvents {
  "order.accepted": OrderAcceptedEventPayload;
}

export interface IEventBus {
  publish<K extends keyof DomainEvents>(event: K, payload: DomainEvents[K]): void;
  subscribe<K extends keyof DomainEvents>(
    event: K,
    handler: (payload: DomainEvents[K]) => void | Promise<void>,
  ): void;
}
