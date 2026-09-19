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

// src/order/infrastructure/sql-order.repository.ts
replaceInFile('order/infrastructure/sql-order.repository.ts', [
  [/return order \? order : null;/g, 'return order!;']
]);

// src/offer/infrastructure/sql-offer.repository.ts
replaceInFile('offer/infrastructure/sql-offer.repository.ts', [
  [/return record \? this.mapToDomain\(record\) : null;/g, 'return this.mapToDomain(record!);']
]);

// src/fulfillment/infrastructure/sql-fulfillment.repository.ts
replaceInFile('fulfillment/infrastructure/sql-fulfillment.repository.ts', [
  [/return fulfillment \? fulfillment : null;/g, 'return fulfillment!;']
]);

console.log("Done fixing non-null assertions.");
