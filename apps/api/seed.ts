import { createDb } from "./src/database/db";
import { accounts, users, sessions, offers } from "./src/database/schema";
import { TokenService } from "./src/auth/infrastructure/services/token.service";
import { config } from "./src/config";

const TEST_USER_ID = "9c630b3d-9641-4a0d-b76f-4e5bdda5ed85";
const TEST_OFFER_ID = "9af24220-9472-4b5d-8889-6a8e914b167f";
const ACCOUNT_ID = "12345678-1234-1234-1234-123456789012";
const SESSION_ID = "87654321-4321-4321-4321-210987654321";
const BEARER_TOKEN = process.env.BEARER_TOKEN;
if (!BEARER_TOKEN) {
  console.error("Environment variable BEARER_TOKEN must be set");
  process.exit(1);
}

const db = createDb(config);
const tokenService = new TokenService(config);

async function seed() {
  console.log("Seeding test data...");

  const hashedToken = tokenService.hash(BEARER_TOKEN);
  const nextMonth = new Date();
  nextMonth.setMonth(nextMonth.getMonth() + 1);

  await db.insert(accounts).values({
    id: ACCOUNT_ID,
    email: "loadtest@example.com",
    emailVerified: true,
  });

  await db.insert(users).values({
    id: TEST_USER_ID,
    accountId: ACCOUNT_ID,
    username: "loadtestuser",
    name: "Load Test User",
  });

  await db.insert(sessions).values({
    id: SESSION_ID,
    accountId: ACCOUNT_ID,
    token: hashedToken,
    expiresAt: nextMonth,
  });

  await db.insert(offers).values({
    id: TEST_OFFER_ID,
    userId: TEST_USER_ID,
    title: "Test Offer",
    description: "This is a test offer for the load test",
    price: 1000,
    currency: "USD",
  });

  console.log("Seeding complete!");
  process.exit(0);
}

seed().catch(console.error);
