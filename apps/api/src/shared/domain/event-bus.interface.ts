import { MessageRecord } from "../../database/schema";

export interface OrderAcceptedEventPayload {
  orderId: string;
  requesterId: string;
  ownerId: string;
}

export interface OrderRequestedEventPayload {
  orderId: string;
  ownerId: string;
}

export interface MessageCreatedEventPayload {
  threadId: string;
  senderId: string;
  receiverId: string;
  message: MessageRecord;
}

export interface DomainEvents {
  "order.accepted": OrderAcceptedEventPayload;
  "order.requested": OrderRequestedEventPayload;
  "message.created": MessageCreatedEventPayload;
}

export interface IEventBus {
  publish<K extends keyof DomainEvents>(event: K, payload: DomainEvents[K]): void;
  subscribe<K extends keyof DomainEvents>(
    event: K,
    handler: (payload: DomainEvents[K]) => void | Promise<void>,
  ): void;
}
