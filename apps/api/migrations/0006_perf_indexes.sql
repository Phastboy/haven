-- no transaction
-- Migration: 0006_perf_indexes
-- Description: Add indexes on hot FK query columns identified during load testing.
--
-- Affected queries:
--   Offer.userId   → GET /api/offers/user/:userId, offer visibility checks in orders
--   Order.requesterId → GET /api/orders/me (buyer view)
--   Order.offerId  → GET /api/orders/received (join with Offer)
--   Order.status   → findPendingByRequesterAndOffer (composite WHERE)
--   Session.expiresAt → deleteExpired (scheduler job)
--
-- All indexes use CONCURRENTLY to avoid table locks (safe during live operations).

CREATE INDEX CONCURRENTLY IF NOT EXISTS "Offer_userId_createdAt_idx"
  ON "Offer" ("userId", "createdAt" DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS "Order_requesterId_idx"
  ON "Order" ("requesterId");

CREATE INDEX CONCURRENTLY IF NOT EXISTS "Order_offerId_idx"
  ON "Order" ("offerId");

-- Composite index for findPendingByRequesterAndOffer
-- Covers: WHERE requesterId = $1 AND offerId = $2 AND status = 'PENDING'
CREATE INDEX CONCURRENTLY IF NOT EXISTS "Order_requesterId_offerId_status_idx"
  ON "Order" ("requesterId", "offerId", "status");

-- Allows efficient deleteExpired() without a seq scan
CREATE INDEX CONCURRENTLY IF NOT EXISTS "Session_expiresAt_idx"
  ON "Session" ("expiresAt");
