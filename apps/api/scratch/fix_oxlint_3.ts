import * as fs from "fs";
import * as path from "path";

const SRC = path.join(__dirname, "../src");

function replaceInFile(filepath: string, replacements: [RegExp | string, string][]) {
  const fullPath = path.join(SRC, filepath);
  if (!fs.existsSync(fullPath)) return;
  let content = fs.readFileSync(fullPath, "utf8");
  for (const [search, replace] of replacements) {
    content = content.replace(search, replace);
  }
  fs.writeFileSync(fullPath, content);
}

replaceInFile("order/application/__tests__/order-use-cases.test.ts", [
  [
    /const mockOrderRepo = \{\} as unknown as import\("\.\.\/\.\.\/domain\/order\.repository"\)\.IOrderRepository = \{\};/g,
    'const mockOrderRepo = {} as unknown as import("../../domain/order.repository").IOrderRepository;',
  ],
]);

console.log("Done fixing syntax error.");
