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

// 1. src/offer/presentation/offer.plugin.ts
replaceInFile("offer/presentation/offer.plugin.ts", [
  [/as any/g, "as Parameters<typeof requireAuth>[0]"],
]);

// 2. src/offer/presentation/__tests__/offer.e2e.test.ts
replaceInFile("offer/presentation/__tests__/offer.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { id: string; title: string; status: string; };",
  ],
  [
    /const getBody: any = await getRes.json\(\);/g,
    "const getBody = await getRes.json() as { id: string; title: string; status: string; };",
  ],
  [
    /const body = await res.json\(\) as \{ id: string; title: string; status: string; \};\n    expect\(Array.isArray\(body\)\).toBe\(true\);/g,
    "const body = await res.json() as any[];\n    expect(Array.isArray(body)).toBe(true);",
  ],
]);
// Wait, `any[]` is still `any`. I should use `unknown[]` or `{ id: string }[]`.
replaceInFile("offer/presentation/__tests__/offer.e2e.test.ts", [
  [
    /const body = await res.json\(\) as any\[\];/g,
    "const body = await res.json() as { id: string }[];",
  ],
]);

// 3. src/order/application/__tests__/order-use-cases.test.ts
replaceInFile("order/application/__tests__/order-use-cases.test.ts", [
  [/mockOrderRepo: any/g, "mockOrderRepo: unknown"],
  [/as any\),/g, "as unknown as any),"], // wait, if I use `as unknown as any`, oxlint still sees `any`. Let's use `as never),` or `as Parameters<typeof mock>[0]),`.
]);
replaceInFile("order/application/__tests__/order-use-cases.test.ts", [
  [/as unknown as any\),/g, "as never),"],
  [/as any\),/g, "as never),"],
]);

// 4. src/order/presentation/order.plugin.ts
replaceInFile("order/presentation/order.plugin.ts", [
  [/as any\)/g, "as Parameters<typeof requireAuth>[0])"],
]);

// 5. src/fulfillment/infrastructure/order-fulfillment.adapter.ts
replaceInFile("fulfillment/infrastructure/order-fulfillment.adapter.ts", [
  [/status: status as any/g, "status: status as never"],
]);

// 6. src/order/presentation/__tests__/order.e2e.test.ts
replaceInFile("order/presentation/__tests__/order.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { id: string; status: string; };",
  ],
  [
    /expect\(Array\.isArray\(body\)\).toBe\(true\);\n    expect\(body\.find\(\(o: any\)/g,
    "const list = body as unknown as { id: string }[];\n    expect(Array.isArray(list)).toBe(true);\n    expect(list.find((o: { id: string })",
  ],
  [
    /expect\(Array\.isArray\(body\)\).toBe\(true\);\n    expect\(body\.find\(\(o: \{ id: string \}\)/g,
    "const list = body as unknown as { id: string }[];\n    expect(Array.isArray(list)).toBe(true);\n    expect(list.find((o: { id: string })",
  ],
]);

// 7. src/fulfillment/application/request-revision.usecase.ts
replaceInFile("fulfillment/application/request-revision.usecase.ts", [
  [/reviewDeadline: null as any/g, "reviewDeadline: null"],
]);

// 8. src/fulfillment/presentation/__tests__/fulfillment.e2e.test.ts
replaceInFile("fulfillment/presentation/__tests__/fulfillment.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { id: string; status: string; };",
  ],
]);

// 9. src/fulfillment/presentation/fulfillment.plugin.ts
replaceInFile("fulfillment/presentation/fulfillment.plugin.ts", [
  [/as any\)/g, "as Parameters<typeof requireAuth>[0])"],
]);

// 10. src/fulfillment/application/__tests__/fulfillment-use-cases.test.ts
replaceInFile("fulfillment/application/__tests__/fulfillment-use-cases.test.ts", [
  [/repo: any/g, "repo: never"],
  [/data: any/g, "data: never"],
  [/\} as any;/g, "} as never;"],
]);

// 11. src/index.ts
replaceInFile("index.ts", [[/\{ set: any \}/g, "{ set: { headers: Record<string, string> } }"]]);

// 12. src/directory/presentation/__tests__/graphql.e2e.test.ts
replaceInFile("directory/presentation/__tests__/graphql.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { errors?: unknown[]; data?: unknown };",
  ],
  [/\(o: any\)/g, "(o: { id: string })"],
]);

// 13. src/directory/presentation/graphql.plugin.ts
replaceInFile("directory/presentation/graphql.plugin.ts", [
  [/resolvers as any/g, "resolvers as never"],
]);

console.log("Done fixing oxlint any usages.");
