import * as fs from 'fs';
import * as path from 'path';

const SRC = path.join(__dirname, '../src');

function replaceInFile(filepath: string, replacements: [RegExp | string, string][]) {
  const fullPath = path.join(SRC, filepath);
  if (!fs.existsSync(fullPath)) return;
  let content = fs.readFileSync(fullPath, 'utf8');
  for (const [search, replace] of replacements) {
    content = content.replace(search, replace);
  }
  fs.writeFileSync(fullPath, content);
}

// 1. src/auth/presentation/__tests__/session.e2e.test.ts
replaceInFile('auth/presentation/__tests__/session.e2e.test.ts', [
  [/import { treaty } from "@elysia\/eden";\n/, '']
]);

// 2. src/auth/presentation/middleware/session.middleware.ts
replaceInFile('auth/presentation/middleware/session.middleware.ts', [
  [/import { Elysia, Context } from "elysia";/, 'import { Context } from "elysia";']
]);

// 3. src/directory/infrastructure/__tests__/sql-directory.repository.integration.test.ts
replaceInFile('directory/infrastructure/__tests__/sql-directory.repository.integration.test.ts', [
  [/import { eq, inArray }/, 'import { inArray }']
]);

// 4. src/directory/presentation/graphql/context.ts
replaceInFile('directory/presentation/graphql/context.ts', [
  [/\n  DirectoryOffer,\n  DirectoryUser,/, '']
]);

// 5. src/fulfillment/application/__tests__/fulfillment-use-cases.test.ts
replaceInFile('fulfillment/application/__tests__/fulfillment-use-cases.test.ts', [
  [/  FulfillmentNotFoundError,\n/, ''],
  [/Mock<\(\) => Promise<{ id: string; status: string }>>/g, 'Mock<() => Promise<{ id: string; status: string; offerId: string; requesterId: string }>>'],
  [/Mock<\(\) => Promise<{ id: string; status: string; offerId: string }>>/g, 'Mock<() => Promise<{ id: string; status: string; offerId: string; requesterId: string }>>'],
  [/Mock<\(\) => Promise<{ id: string; status: string; requesterId: string }>>/g, 'Mock<() => Promise<{ id: string; status: string; offerId: string; requesterId: string }>>']
]);

// 6. src/fulfillment/application/deliver-fulfillment.usecase.ts
replaceInFile('fulfillment/application/deliver-fulfillment.usecase.ts', [
  [/\n  FulfillmentNotFoundError,/, ''],
  [/deliveryMessage: params.deliveryMessage,/, '...(params.deliveryMessage && { deliveryMessage: params.deliveryMessage }),']
]);

// 7. src/fulfillment/infrastructure/__tests__/sql-fulfillment.repository.integration.test.ts
replaceInFile('fulfillment/infrastructure/__tests__/sql-fulfillment.repository.integration.test.ts', [
  [/import { accounts, offers, orders, users, fulfillments }/, 'import { accounts, offers, orders, users }'],
  [/import { eq, inArray }/, 'import { eq }']
]);

// 8. src/fulfillment/presentation/__tests__/fulfillment.e2e.test.ts
replaceInFile('fulfillment/presentation/__tests__/fulfillment.e2e.test.ts', [
  [/import { users, accounts, sessions, offers, orders, fulfillments }/, 'import { users, accounts, sessions, offers, orders }'],
  [/expect\(order.status\)/, 'expect(order!.status)']
]);

// 9. src/fulfillment/presentation/fulfillment.plugin.ts
replaceInFile('fulfillment/presentation/fulfillment.plugin.ts', [
  [/import { Elysia, t }/, 'import { Elysia }'],
  [/user, session, set/, 'session, set'],
  [/deliveryMessage: body.deliveryMessage,/, '...(body.deliveryMessage && { deliveryMessage: body.deliveryMessage }),']
]);

// 10. src/index.ts
replaceInFile('index.ts', [
  [/.options\("\/\*", \(\{ set \}\) => \{/, '.options("/*", ({ set }: { set: any }) => {']
]);

// 11. src/offer/infrastructure/sql-offer.repository.ts
replaceInFile('offer/infrastructure/sql-offer.repository.ts', [
  [/return this.mapToDomain\(record\);/, 'return record ? this.mapToDomain(record) : undefined;']
]);

// 12. src/offer/presentation/__tests__/offer.e2e.test.ts
replaceInFile('offer/presentation/__tests__/offer.e2e.test.ts', [
  [/import { accounts, users, sessions, offers }/, 'import { accounts, users, sessions }']
]);

// 13. src/order/application/__tests__/order-use-cases.test.ts
replaceInFile('order/application/__tests__/order-use-cases.test.ts', [
  [/\n  OrderNotFoundError,/, '']
]);

// 14. src/order/application/create-order.usecase.ts
replaceInFile('order/application/create-order.usecase.ts', [
  [/message: params.message,/, '...(params.message && { message: params.message }),']
]);

// 15. src/order/infrastructure/__tests__/sql-order.repository.integration.test.ts
replaceInFile('order/infrastructure/__tests__/sql-order.repository.integration.test.ts', [
  [/import { accounts, offers, orders, users }/, 'import { accounts, offers, users }'],
  [/import { eq, inArray }/, 'import { inArray }']
]);

// 16. src/order/infrastructure/sql-order.repository.ts
replaceInFile('order/infrastructure/sql-order.repository.ts', [
  [/import { eq, inArray }/, 'import { eq }'],
  [/import { orders, offers, users }/, 'import { orders, offers }']
]);

// 17. src/order/presentation/__tests__/order.e2e.test.ts
replaceInFile('order/presentation/__tests__/order.e2e.test.ts', [
  [/import { users, accounts, sessions, offers, orders }/, 'import { users, accounts, sessions, offers }'],
  [/import { eq, inArray }/, 'import { inArray }']
]);

// 18. src/order/presentation/order.plugin.ts
replaceInFile('order/presentation/order.plugin.ts', [
  [/message: body.message,/, '...(body.message && { message: body.message }),']
]);

// 19. src/user/infrastructure/__tests__/sql-user.repository.integration.test.ts
replaceInFile('user/infrastructure/__tests__/sql-user.repository.integration.test.ts', [
  [/import { NotFoundError } from "\.\.\/\.\.\/\.\.\/shared\/errors";\n/, '']
]);

// 20. src/user/presentation/__tests__/user.e2e.test.ts
replaceInFile('user/presentation/__tests__/user.e2e.test.ts', [
  [/import { treaty } from "@elysia\/eden";\n/, '']
]);

// 21. src/user/presentation/user.plugin.ts
replaceInFile('user/presentation/user.plugin.ts', [
  [/import { CreateUserUseCase } from "\.\.\/application\/create-user.usecase";\n/, ''],
  [/import { DeleteUserUseCase } from "\.\.\/application\/delete-user.usecase";\n/, '']
]);

// Also fix Static in offer.dto.ts not being used
replaceInFile('offer/presentation/offer.dto.ts', [
  [/import type { Static } from "typebox";\n/, '']
]);

console.log("Done fixing TS errors.");
