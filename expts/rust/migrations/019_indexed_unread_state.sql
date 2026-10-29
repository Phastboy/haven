-- Cache the existing latest-sender/read predicate, rather than scanning every
-- conversation when rendering an authenticated page. The migration transaction
-- holds the table locks through backfill and trigger installation.
LOCK TABLE conversation, message, conversation_participant IN ACCESS EXCLUSIVE MODE;

ALTER TABLE conversation_participant
    ADD COLUMN is_unread BOOLEAN NOT NULL DEFAULT FALSE;

-- NOW() is the transaction start, which may precede a wait for a concurrent
-- read/send. Assign message time after the conversation lock is acquired.
ALTER TABLE message ALTER COLUMN created_at SET DEFAULT clock_timestamp();

CREATE FUNCTION derive_participant_unread() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    NEW.is_unread := COALESCE((
        SELECT NEW.last_read_at < c.last_message_at AND latest.sender_id <> NEW.user_id
        FROM conversation c
        JOIN LATERAL (
            SELECT m.sender_id FROM message m
            WHERE m.conversation_id = c.id
            ORDER BY m.created_at DESC, m.id DESC LIMIT 1
        ) latest ON TRUE
        WHERE c.id = NEW.conversation_id
    ), FALSE);
    RETURN NEW;
END;
$$;

CREATE TRIGGER participant_unread_on_read
BEFORE INSERT OR UPDATE OF last_read_at ON conversation_participant
FOR EACH ROW EXECUTE FUNCTION derive_participant_unread();

CREATE FUNCTION refresh_conversation_unread() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    -- Re-evaluate both participants after the message and activity timestamp
    -- have been written. Bulk fixtures use the same timestamp update.
    UPDATE conversation_participant SET last_read_at = last_read_at
    WHERE conversation_id = NEW.id;
    RETURN NEW;
END;
$$;

CREATE TRIGGER conversation_unread_on_activity
AFTER UPDATE OF last_message_at ON conversation
FOR EACH ROW EXECUTE FUNCTION refresh_conversation_unread();

-- Fire the derivation for existing participants, including empty conversations.
UPDATE conversation_participant SET last_read_at = last_read_at;

CREATE INDEX idx_participant_unread_user
    ON conversation_participant (user_id) WHERE is_unread;
