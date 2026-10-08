-- Backfill any existing NULL or too-short descriptions
UPDATE offer
SET description = 'No description provided for legacy offer.'
WHERE description IS NULL OR char_length(trim(description)) < 10;

-- Enforce NOT NULL on description
ALTER TABLE offer ALTER COLUMN description SET NOT NULL;

-- Enforce character length constraints on title (3 to 100 characters)
ALTER TABLE offer ADD CONSTRAINT check_offer_title_length
    CHECK (char_length(trim(title)) >= 3 AND char_length(trim(title)) <= 100);

-- Enforce character length constraints on description (10 to 2000 characters)
ALTER TABLE offer ADD CONSTRAINT check_offer_description_length
    CHECK (char_length(trim(description)) >= 10 AND char_length(trim(description)) <= 2000);

-- Replace generic 3-uppercase currency check with Haven whitelist
ALTER TABLE offer DROP CONSTRAINT IF EXISTS check_currency_format;
ALTER TABLE offer ADD CONSTRAINT check_currency_whitelist
    CHECK (currency IN ('NGN', 'USD', 'EUR', 'GBP', 'CAD', 'AUD', 'KES', 'GHS'));
