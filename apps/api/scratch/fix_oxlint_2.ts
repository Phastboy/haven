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

// 1. Session Middleware (requireAuth fix)
replaceInFile("auth/presentation/middleware/session.middleware.ts", [
  [
    /context: Context & \{ session\?: SessionWithAccount \| null \}/,
    "context: { session?: SessionWithAccount | null; set: { status?: number | string; [key: string]: unknown } }",
  ],
]);

// 2. Plugins (remove as any for requireAuth)
const pluginFiles = [
  "offer/presentation/offer.plugin.ts",
  "order/presentation/order.plugin.ts",
  "fulfillment/presentation/fulfillment.plugin.ts",
];
for (const file of pluginFiles) {
  replaceInFile(file, [[/\{ session, set \} as any/g, "{ session, set }"]]);
}

// 3. E2E Tests (replace `const body: any = await res.json()`)
replaceInFile("offer/presentation/__tests__/offer.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { id?: string; title?: string; status?: string; userId?: string; [key: string]: unknown };",
  ],
  [
    /const getBody: any = await getRes.json\(\);/g,
    "const getBody = await getRes.json() as { id?: string; title?: string; status?: string; userId?: string; [key: string]: unknown };",
  ],
  [/\(o: any\)/g, "(o: { id: string })"],
]);

replaceInFile("order/presentation/__tests__/order.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { id?: string; status?: string; price?: number; quantity?: number; [key: string]: unknown };",
  ],
  [/\(o: any\)/g, "(o: { id: string })"],
]);

replaceInFile("fulfillment/presentation/__tests__/fulfillment.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { id?: string; status?: string; deliveryMessage?: string; [key: string]: unknown };",
  ],
]);

replaceInFile("directory/presentation/__tests__/graphql.e2e.test.ts", [
  [
    /const body: any = await res.json\(\);/g,
    "const body = await res.json() as { errors?: unknown[]; data?: { activeOffers?: { id: string; [key: string]: unknown }[] } };",
  ],
  [/\(o: any\)/g, "(o: { id: string })"],
]);

// 4. Use Cases Tests (replace mock: any)
replaceInFile("order/application/__tests__/order-use-cases.test.ts", [
  [
    /mockOrderRepo: any/g,
    'mockOrderRepo = {} as unknown as import("../../domain/order.repository").IOrderRepository',
  ],
  [/as any\),/g, 'as unknown as import("../../domain/order.schema").Order),'],
]);

replaceInFile("fulfillment/application/__tests__/fulfillment-use-cases.test.ts", [
  [
    /const repo: any = \{\};/g,
    'const repo = {} as unknown as import("../../domain/fulfillment.repository").IFulfillmentRepository;',
  ],
  [
    /const repo = \{\} as any;/g,
    'const repo = {} as unknown as import("../../domain/fulfillment.repository").IFulfillmentRepository;',
  ],
  [/mock\(async \(data: any\) =>/g, "mock(async (data: { id: string, orderId: string }) =>"],
]);

// 5. Adapters and Plugins
replaceInFile("fulfillment/infrastructure/order-fulfillment.adapter.ts", [
  [
    /status: status as any/g,
    'status: status as "PENDING" | "ACCEPTED" | "REJECTED" | "COMPLETED" | "CANCELLED"',
  ],
]);

replaceInFile("fulfillment/application/request-revision.usecase.ts", [
  [/reviewDeadline: null as any/g, "reviewDeadline: null as unknown as Date"],
]);

replaceInFile("index.ts", [[/\{ set: any \}/g, "{ set: { headers: Record<string, string> } }"]]);

replaceInFile("directory/presentation/graphql.plugin.ts", [
  [/resolvers as any/g, "resolvers as unknown"],
]);

console.log("Done fixing TS and oxlint.");
