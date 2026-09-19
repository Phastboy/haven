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

// 1. auth/presentation/middleware/session.middleware.ts
replaceInFile("auth/presentation/middleware/session.middleware.ts", [
  [/import { Context } from "elysia";\n/, ""],
]);

// 2. directory/presentation/__tests__/graphql.e2e.test.ts
replaceInFile("directory/presentation/__tests__/graphql.e2e.test.ts", [
  [
    /data\?: \{ activeOffers\?: \{ id: string; \[key: string\]: unknown \}\[\] \}/,
    "data?: { activeOffers?: { id: string; title: string; price: number; user: { id: string; username: string; } }[] }",
  ],
  [
    /const fetchedOffers = body\.data\.activeOffers;/,
    "const fetchedOffers = body.data!.activeOffers!;",
  ],
]);

// 3. directory/presentation/graphql.plugin.ts
replaceInFile("directory/presentation/graphql.plugin.ts", [
  [/resolvers as unknown,/g, "resolvers as never,"],
]);

// 4. offer/presentation/__tests__/offer.e2e.test.ts
replaceInFile("offer/presentation/__tests__/offer.e2e.test.ts", [
  [/offerId1 = body\.id;/, "offerId1 = body.id!;"],
  [
    /const body = await res\.json\(\) as \{ \[key: string\]: unknown \};/g,
    "const body = await res.json() as unknown[];",
  ], // Wait, the regex wasn't like this.
  [
    /const body = await res.json\(\) as \{ id\?: string; title\?: string; status\?: string; userId\?: string; \[key: string\]: unknown \};\n    expect\(Array\.isArray\(body\)\)\.toBe\(true\);/g,
    "const body = await res.json() as { id: string; title: string; }[];\n    expect(Array.isArray(body)).toBe(true);",
  ],
]);

// 5. order/presentation/__tests__/order.e2e.test.ts
replaceInFile("order/presentation/__tests__/order.e2e.test.ts", [
  [/testOrderId = body\.id;/, "testOrderId = body.id!;"],
  [
    /const body = await res.json\(\) as \{ id\?: string; status\?: string; price\?: number; quantity\?: number; \[key: string\]: unknown \};\n    expect\(Array\.isArray\(body\)\)\.toBe\(true\);/g,
    "const body = await res.json() as { id: string; status: string; }[];\n    expect(Array.isArray(body)).toBe(true);",
  ],
]);

console.log("Done fixing the rest of TS errors.");
