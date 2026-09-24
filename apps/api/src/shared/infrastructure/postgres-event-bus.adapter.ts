import postgres from "postgres";
import type { IEventBus, DomainEvents } from "../domain/event-bus.interface";

export class PostgresEventBusAdapter implements IEventBus {
  private sql: postgres.Sql;
  private listeners: Map<string, ((payload: unknown) => void)[]> = new Map();

  constructor(connectionString: string) {
    // We use a dedicated connection for LISTEN/NOTIFY.
    this.sql = postgres(connectionString, {
      max: 1, // Only need 1 connection for the listener
      onclose: () => {
        console.warn("[PostgresEventBus] Connection closed.");
      },
    });
  }

  async publish<K extends keyof DomainEvents>(event: K, payload: DomainEvents[K]): Promise<void> {
    try {
      const payloadStr = JSON.stringify(payload);
      // notify channel name max length is 63 chars, which is fine for our events.
      await this.sql`SELECT pg_notify(${event}, ${payloadStr})`;
    } catch (err) {
      console.error(`[PostgresEventBusAdapter] Error publishing event ${event}:`, err);
    }
  }

  async subscribe<K extends keyof DomainEvents>(
    event: K,
    handler: (payload: DomainEvents[K]) => void | Promise<void>,
  ): Promise<void> {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
      // Setup the PG listen once per event channel
      try {
        await this.sql.listen(event, async (payloadStr: string) => {
          let payload: DomainEvents[K];
          try {
            payload = JSON.parse(payloadStr) as DomainEvents[K];
          } catch (err) {
            console.error(`[PostgresEventBus] Failed to parse payload for event ${event}:`, err);
            return;
          }
          const handlers = this.listeners.get(event) || [];
          for (const h of handlers) {
            try {
              await h(payload);
            } catch (handlerErr) {
              console.error(`[PostgresEventBus] Handler failed for event ${event}:`, handlerErr);
            }
          }
        });
      } catch (listenErr) {
        this.listeners.delete(event);
        throw listenErr;
      }
    }

    this.listeners.get(event)!.push(handler as unknown as (payload: unknown) => void);
  }
}
