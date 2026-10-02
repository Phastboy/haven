-- Strategic indexes for foreign keys to optimize joins, filters, and cascade deletes
-- Account constraint creates a unique index on "user"(account_id).
-- Token constraint creates a unique index on magic_link(token_hash).
-- Token constraint creates a unique index on session(token_hash).

-- 1. Index on offer.user_id (for fast lookups of a user's offers and for cascading deletes)
CREATE INDEX idx_offer_user_id ON offer(user_id);

-- 2. Index on session.account_id (for looking up user's active sessions and cascading deletes)
CREATE INDEX idx_session_account_id ON session(account_id);
