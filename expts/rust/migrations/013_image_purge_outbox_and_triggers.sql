-- 013_image_purge_outbox_and_triggers.sql: Transactional outbox and engine triggers for image deletion

-- 1. Image purge outbox table for reliable external asset cleanup
CREATE TABLE image_purge_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_id VARCHAR(120) NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_image_purge_outbox_pending 
    ON image_purge_outbox (created_at ASC) 
    WHERE attempts < 5;

-- 2. Trigger for offer_image deletion (covers single image delete AND cascading offer delete)
CREATE OR REPLACE FUNCTION trg_offer_image_purge_outbox()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO image_purge_outbox (public_id) VALUES (OLD.public_id);
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER offer_image_after_delete_purge
    AFTER DELETE ON offer_image
    FOR EACH ROW
    EXECUTE FUNCTION trg_offer_image_purge_outbox();

-- 3. Trigger for user avatar change, clearing, or deletion
CREATE OR REPLACE FUNCTION trg_user_avatar_purge_outbox()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.avatar_public_id IS NOT NULL THEN
            INSERT INTO image_purge_outbox (public_id) VALUES (OLD.avatar_public_id);
        END IF;
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.avatar_public_id IS NOT NULL AND (NEW.avatar_public_id IS DISTINCT FROM OLD.avatar_public_id) THEN
            INSERT INTO image_purge_outbox (public_id) VALUES (OLD.avatar_public_id);
        END IF;
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_avatar_after_update_purge
    AFTER UPDATE OR DELETE ON "user"
    FOR EACH ROW
    EXECUTE FUNCTION trg_user_avatar_purge_outbox();

-- 4. Partial composite index for public profile feeds (only offers with image_count >= 1)
CREATE INDEX idx_offer_user_feed_visible 
    ON offer (user_id, created_at DESC, id DESC) 
    WHERE image_count >= 1;
