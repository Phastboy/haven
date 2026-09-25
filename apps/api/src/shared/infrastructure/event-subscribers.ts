import type { App } from "../../index";
import type { PostgresEventBusAdapter } from "./postgres-event-bus.adapter";
import { SqlMessageRepository } from "../../message/infrastructure/sql-message.repository";
import { CreateThreadUseCase } from "../../message/application/create-thread.usecase";

import type { DB } from "../../database/db";

/**
 * Sets up all cross-domain event listeners that bridge different bounded contexts.
 */
export async function setupEventSubscribers(eventBus: PostgresEventBusAdapter, app: App, db: DB) {
  // 1. Thread creation on order acceptance
  await eventBus.subscribe("order.accepted", async (payload) => {
    const messageRepo = new SqlMessageRepository(db);
    const createThreadUseCase = new CreateThreadUseCase(messageRepo);
    try {
      await createThreadUseCase.execute(payload.requesterId, payload.ownerId);
      console.log(`[EventBus] Thread auto-created for order ${payload.orderId}`);
    } catch (error) {
      console.error(`[EventBus] Error creating thread for order ${payload.orderId}:`, error);
    }
  });

  // 2. Real-time notifications for order requests
  await eventBus.subscribe("order.requested", (payload) => {
    app.server?.publish(
      `user:${payload.ownerId}`,
      JSON.stringify({ type: "NOTIFICATION", data: { message: "Someone requested your offer!" } }),
    );
  });

  // 3. Real-time notifications for order acceptances
  await eventBus.subscribe("order.accepted", (payload) => {
    app.server?.publish(
      `user:${payload.requesterId}`,
      JSON.stringify({ type: "NOTIFICATION", data: { message: "Your request was accepted!" } }),
    );
  });

  // 4. Real-time notifications for new messages
  await eventBus.subscribe("message.created", (payload) => {
    app.server?.publish(
      `user:${payload.receiverId}`,
      JSON.stringify({ type: "NEW_MESSAGE", data: payload.message }),
    );
  });
}
