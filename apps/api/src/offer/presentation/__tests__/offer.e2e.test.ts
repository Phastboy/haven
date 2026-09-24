import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { app } from "../../../index";
import { db } from "../../../database/db";
import { accounts, users, sessions } from "../../../database/schema";
import { randomUUID } from "crypto";
import { eq } from "drizzle-orm";
import { tokenService } from "../../../auth/infrastructure/services/token.service";

describe("Offer Plugin E2E", () => {
  let testAccountId1: string;
  let testUserId1: string;
  let rawToken1: string;

  let testAccountId2: string;
  let testUserId2: string;
  let rawToken2: string;

  let offerId1: string;

  beforeAll(async () => {
    testAccountId1 = randomUUID();
    testUserId1 = randomUUID();
    rawToken1 = tokenService.generate();

    await db.insert(accounts).values({
      id: testAccountId1,
      email: `offer-user-1-${Date.now()}@example.com`,
    });
    await db.insert(users).values({
      id: testUserId1,
      accountId: testAccountId1,
      username: `offer1-${Date.now()}`,
    });
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: testAccountId1,
      token: tokenService.hash(rawToken1),
      expiresAt: new Date(Date.now() + 1000 * 60 * 60),
    });

    testAccountId2 = randomUUID();
    testUserId2 = randomUUID();
    rawToken2 = tokenService.generate();

    await db.insert(accounts).values({
      id: testAccountId2,
      email: `offer-user-2-${Date.now()}@example.com`,
    });
    await db.insert(users).values({
      id: testUserId2,
      accountId: testAccountId2,
      username: `offer2-${Date.now()}`,
    });
    await db.insert(sessions).values({
      id: randomUUID(),
      accountId: testAccountId2,
      token: tokenService.hash(rawToken2),
      expiresAt: new Date(Date.now() + 1000 * 60 * 60),
    });
  });

  afterAll(async () => {
    await db.delete(accounts).where(eq(accounts.id, testAccountId1));
    await db.delete(accounts).where(eq(accounts.id, testAccountId2));
  });

  it("should create an offer successfully (POST /offers)", async () => {
    const req = new Request("http://localhost/api/offers", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${rawToken1}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        title: "E2E Offer",
        price: 1500,
        offerType: "SERVICE",
      }),
    });

    const res = await app.handle(req);
    if (res.status !== 201) {
      console.error(await res.text());
    }
    expect(res.status).toBe(201);
    const body = (await res.json()) as {
      data?: {
        id?: string;
        title?: string;
        status?: string;
        userId?: string;
        [key: string]: unknown;
      };
    };
    expect(body.data?.title).toBe("E2E Offer");
    expect(body.data?.userId).toBe(testUserId1);
    offerId1 = body.data!.id!;
  });

  it("should prevent unauthorized users from editing offers (PATCH /offers/:id)", async () => {
    const req = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "PATCH",
      headers: {
        Authorization: `Bearer ${rawToken2}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        title: "Hacked Title",
      }),
    });

    const res = await app.handle(req);
    expect(res.status).toBe(403);
  });

  it("should allow the owner to update the offer (PATCH /offers/:id)", async () => {
    const req = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "PATCH",
      headers: {
        Authorization: `Bearer ${rawToken1}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        title: "Updated Title E2E",
      }),
    });

    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as {
      data?: {
        id?: string;
        title?: string;
        status?: string;
        userId?: string;
        [key: string]: unknown;
      };
    };
    expect(body.data?.title).toBe("Updated Title E2E");
  });

  it("should retrieve a public offer (GET /offers/:id)", async () => {
    const req = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "GET",
    });

    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as {
      data?: {
        id?: string;
        title?: string;
        status?: string;
        userId?: string;
        [key: string]: unknown;
      };
    };
    expect(body.data?.title).toBe("Updated Title E2E");
  });

  it("should list offers for a user publicly (GET /offers/user/:userId)", async () => {
    const req = new Request(`http://localhost/api/offers/user/${testUserId1}`, {
      method: "GET",
    });

    const res = await app.handle(req);
    expect(res.status).toBe(200);
    const body = (await res.json()) as { data: { id: string; title: string }[] };
    expect(Array.isArray(body.data)).toBe(true);
    expect(body.data.length).toBeGreaterThan(0);
    expect(body.data[0]!.title).toBe("Updated Title E2E");
  });

  it("should prevent unauthorized users from deleting (DELETE /offers/:id)", async () => {
    const req = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${rawToken2}`,
      },
    });

    const res = await app.handle(req);
    expect(res.status).toBe(403);
  });

  it("should allow the owner to delete the offer (DELETE /offers/:id)", async () => {
    const req = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${rawToken1}`,
      },
    });

    const res = await app.handle(req);
    expect(res.status).toBe(204);

    // Archived offer is invisible to unauthenticated callers — expect 404
    const getReq = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "GET",
    });
    const getRes = await app.handle(getReq);
    expect(getRes.status).toBe(404);

    // But the owner can still see it (it's a soft-delete / archive)
    const ownerGetReq = new Request(`http://localhost/api/offers/${offerId1}`, {
      method: "GET",
      headers: { Authorization: `Bearer ${rawToken1}` },
    });
    const ownerGetRes = await app.handle(ownerGetReq);
    expect(ownerGetRes.status).toBe(200);
    const getBody = (await ownerGetRes.json()) as {
      data?: {
        id?: string;
        title?: string;
        status?: string;
        userId?: string;
        [key: string]: unknown;
      };
    };
    expect(getBody.data?.status).toBe("ARCHIVED");
  });
});
