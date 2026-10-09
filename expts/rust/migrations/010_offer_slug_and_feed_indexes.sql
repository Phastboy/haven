-- 1. Add slug column with unique random default for zero-downtime inserts during migration rollout
ALTER TABLE offer ADD COLUMN slug VARCHAR(120) NOT NULL DEFAULT ('offer-' || substr(gen_random_uuid()::text, 1, 8));

-- 2. Backfill existing rows with clean URL-safe slug derived from title and unique ID prefix.
-- Trims trailing hyphens after truncating to 80 chars, falling back to 'offer' if empty,
-- matching Rust's OfferSlug::from_title_and_suffix and preventing doubled hyphens.
UPDATE offer
SET slug = COALESCE(NULLIF(rtrim(left(COALESCE(NULLIF(lower(regexp_replace(regexp_replace(trim(title), '[^a-zA-Z0-9]+', '-', 'g'), '^-+|-+$', '', 'g')), ''), 'offer'), 80), '-'), ''), 'offer') || '-' || substr(id::text, 1, 8)
WHERE slug LIKE 'offer-%';

-- 3. Remove temporary default so future inserts must supply a valid slug
ALTER TABLE offer ALTER COLUMN slug DROP DEFAULT;

-- 4. Enforce valid URL slug format (lowercase alphanumeric and hyphens, no consecutive/trailing hyphens)
ALTER TABLE offer ADD CONSTRAINT check_offer_slug_format
    CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(slug) >= 3 AND char_length(slug) <= 120);

-- 5. Create unique index on slug for fast public lookups (/offers/{slug})
CREATE UNIQUE INDEX idx_offer_slug ON offer (slug);

-- 6. Create composite keyset pagination indexes for stable, constant-time feed traversal
CREATE INDEX idx_offer_feed ON offer (created_at DESC, id DESC);
CREATE INDEX idx_offer_user_feed ON offer (user_id, created_at DESC, id DESC);

