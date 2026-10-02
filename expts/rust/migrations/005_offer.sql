-- Offer: something a platform participant is willing to provide (links to User)
CREATE TABLE offer (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    title       VARCHAR(255) NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT,
    price       INTEGER CHECK (price IS NULL OR price >= 0),
    currency    CHAR(3),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT currency_required_when_priced
        CHECK (price IS NULL OR price = 0 OR currency IS NOT NULL)
);
