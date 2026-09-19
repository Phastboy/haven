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

// 1. context.ts
replaceInFile('directory/presentation/graphql/context.ts', [
  [/import {\n  IDirectoryRepository,\n} from "\.\.\/\.\.\/domain\/directory\.repository";/, 'import { IDirectoryRepository } from "../../domain/directory.repository";\nimport { DirectoryUser } from "../../domain/directory.schema";']
]);

// 2. fulfillment-use-cases.test.ts
replaceInFile('fulfillment/application/__tests__/fulfillment-use-cases.test.ts', [
  [/id: "o1", status: "PENDING" /g, 'id: "o1", status: "PENDING", requesterId: "req1", offerId: "off1" '],
  [/id: "o1", status: "ACCEPTED", offerId: "off1" /g, 'id: "o1", status: "ACCEPTED", offerId: "off1", requesterId: "req1" '],
  [/id: "o1", status: "ACCEPTED", requesterId: "req1" /g, 'id: "o1", status: "ACCEPTED", requesterId: "req1", offerId: "off1" '],
  [/id: "o1",\n          status: "ACCEPTED",\n          requesterId: "req1",\n          offerId: "off1",/g, 'id: "o1", status: "ACCEPTED", requesterId: "req1", offerId: "off1"']
]);

// 3. sql-fulfillment.repository.integration.test.ts
replaceInFile('fulfillment/infrastructure/__tests__/sql-fulfillment.repository.integration.test.ts', [
  [/updated\.reviewDeadline\?\.getTime\(\)/g, 'new Date(updated.reviewDeadline!).getTime()']
]);

console.log("Done fixing TS part 4.");
