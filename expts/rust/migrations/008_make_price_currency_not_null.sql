-- Remove old check constraint
ALTER TABLE offer DROP CONSTRAINT currency_required_when_priced;

-- Backfill existing data
UPDATE offer SET currency = 'NGN' WHERE currency IS NULL;
UPDATE offer SET price = 0 WHERE price IS NULL;

-- Enforce invariants
ALTER TABLE offer ALTER COLUMN currency SET NOT NULL;
ALTER TABLE offer ADD CONSTRAINT check_currency_format CHECK (currency ~ '^[A-Z]{3}$');

ALTER TABLE offer ALTER COLUMN price SET NOT NULL;
ALTER TABLE offer ADD CONSTRAINT check_price_non_negative CHECK (price >= 0);
