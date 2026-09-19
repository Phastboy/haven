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

// 1. src/directory/presentation/graphql/context.ts
replaceInFile('directory/presentation/graphql/context.ts', [
  [/import { eq } from "drizzle-orm";/, '']
]);

// 2. src/fulfillment/application/__tests__/fulfillment-use-cases.test.ts
// Fix the Mock return types for getOrderDetails
replaceInFile('fulfillment/application/__tests__/fulfillment-use-cases.test.ts', [
  [/Mock<\(\) => Promise<{ id: string; status: string; offerId: string; requesterId: string }>>/g, 'Mock<() => Promise<{ id: string; status: string; offerId: string; requesterId: string } | null>>']
]);

// 3. src/fulfillment/infrastructure/__tests__/sql-fulfillment.repository.integration.test.ts
replaceInFile('fulfillment/infrastructure/__tests__/sql-fulfillment.repository.integration.test.ts', [
  [/updated.reviewDeadline\?.getTime\(\)/, 'new Date(updated.reviewDeadline!).getTime()']
]);

// 4. src/fulfillment/presentation/fulfillment.plugin.ts
replaceInFile('fulfillment/presentation/fulfillment.plugin.ts', [
  [/\.\.\.\(body.deliveryMessage && \{ deliveryMessage: body.deliveryMessage \}\),/g, 'deliveryMessage: body.deliveryMessage,']
]);

console.log("Done fixing remaining TS errors.");
