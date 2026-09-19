import { Elysia } from "elysia";
import { cron } from "@elysia/cron";
import { AutoCompleteExpiredUseCase } from "../fulfillment/application/auto-complete-expired.usecase";
import { SqlFulfillmentRepository } from "../fulfillment/infrastructure/sql-fulfillment.repository";
import { OrderFulfillmentAdapter } from "../fulfillment/infrastructure/order-fulfillment.adapter";

export const createAutoCompletePlugin = () => {
  const repository = new SqlFulfillmentRepository();
  const adapter = new OrderFulfillmentAdapter();
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
