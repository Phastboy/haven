import type { DB } from "../database/db";
import { Elysia, type AnyElysia } from "elysia";
import { cron } from "@elysia/cron";
import { AutoCompleteExpiredUseCase } from "../fulfillment/application/auto-complete-expired.usecase";
import { SqlFulfillmentRepository } from "../fulfillment/infrastructure/sql-fulfillment.repository";
import { OrderFulfillmentAdapter } from "../fulfillment/infrastructure/order-fulfillment.adapter";

// The return type is annotated on purpose. Without it TypeScript tries to name
// croner's `Cron` type in the inferred type, gives up ("cannot be named without a
// reference to 'Cron' ... not portable"), and falls back to `any` — which then
// collapses the WHOLE app type at the `.use()` call site and kills Eden's types.
// This plugin registers no routes, so `Elysia` is all the caller needs to know.
export const createAutoCompletePlugin = (db: DB): AnyElysia => {
  const repository = new SqlFulfillmentRepository(db);
  const adapter = new OrderFulfillmentAdapter(db);
  const autoCompleteUseCase = new AutoCompleteExpiredUseCase(repository, adapter);

  return new Elysia({ name: "cron.autoComplete" }).use(
    cron({
      name: "auto-complete-expired-fulfillments",
      pattern: "0 * * * *", // Run at minute 0 of every hour
      async run() {
        console.log(`[Cron] Running auto-complete job...`);
        try {
          const count = await autoCompleteUseCase.execute();
          if (count > 0) {
            console.log(`[Cron] Auto-completed ${count} expired fulfillments.`);
          }
        } catch (err) {
          console.error(`[Cron] Error during auto-complete job:`, err);
        }
      },
    }),
  );
};
