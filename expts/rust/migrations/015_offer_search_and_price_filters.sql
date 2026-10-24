-- Migration 015: Offer title search and price narrowing indexes
--
-- Adds a stored tsvector generated column for full-text title search
-- and partial indexes for title search and price/currency filtering.
-- Both new indexes are partial on image_count >= 1 (matching the public visibility rule).

ALTER TABLE offer
    ADD COLUMN IF NOT EXISTS title_search tsvector
    GENERATED ALWAYS AS (to_tsvector('english', title)) STORED;

CREATE INDEX IF NOT EXISTS idx_offer_title_search
    ON offer USING gin (title_search)
    WHERE image_count >= 1;

CREATE INDEX IF NOT EXISTS idx_offer_feed_currency_price
    ON offer (currency, price, created_at DESC, id DESC)
    WHERE image_count >= 1;
