-- Add idempotency_key to offer table
ALTER TABLE offer ADD COLUMN idempotency_key UUID;

-- Create unique constraint for idempotency
CREATE UNIQUE INDEX idx_offer_user_id_idempotency ON offer(user_id, idempotency_key);

-- Note: idx_offer_user_id was already created in 006_indexes.sql.
