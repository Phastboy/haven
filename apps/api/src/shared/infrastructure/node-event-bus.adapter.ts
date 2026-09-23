import { EventEmitter } from "events";
import type { IEventBus, DomainEvents } from "../domain/event-bus.interface";

export class NodeEventEmitterAdapter implements IEventBus {
  private emitter = new EventEmitter();

  publish<K extends keyof DomainEvents>(event: K, payload: DomainEvents[K]): void {
    this.emitter.emit(event, payload);
  }

  subscribe<K extends keyof DomainEvents>(
    event: K,
    handler: (payload: DomainEvents[K]) => void | Promise<void>,
  ): void {
    this.emitter.on(event, handler);
  }
}
