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
    const payloadStr = JSON.stringify(payload);
    // notify channel name max length is 63 chars, which is fine for our events.
    await this.sql`SELECT pg_notify(${event}, ${payloadStr})`;
  }

  async subscribe<K extends keyof DomainEvents>(
    event: K,
    handler: (payload: DomainEvents[K]) => void | Promise<void>,
  ): Promise<void> {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
      // Setup the PG listen once per event channel
      await this.sql.listen(event, (payloadStr: string) => {
        try {
          const payload = JSON.parse(payloadStr) as DomainEvents[K];
          const handlers = this.listeners.get(event) || [];
          for (const h of handlers) {
            h(payload);
          }
        } catch (err) {
          console.error(
            `[PostgresEventBus] Failed to parse or handle payload for event ${event}:`,
            err,
          );
        }
      });
    }

    this.listeners.get(event)!.push(handler as unknown as (payload: unknown) => void);
  }
}
