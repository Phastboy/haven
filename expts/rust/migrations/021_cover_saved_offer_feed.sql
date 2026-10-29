-- Keep the Saved feed's ordered membership scan index-only.
DROP INDEX idx_saved_offer_user_created;

CREATE INDEX idx_saved_offer_user_created
    ON saved_offer (user_id, created_at DESC, id DESC)
    INCLUDE (offer_id);
