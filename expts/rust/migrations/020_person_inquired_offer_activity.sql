-- Maintain the latest qualifying sent-message activity per person and offer.
LOCK TABLE conversation, message, offer IN ACCESS EXCLUSIVE MODE;

CREATE TABLE person_inquired_offer_activity (
    person_id       UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    offer_id        UUID NOT NULL REFERENCES offer(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL REFERENCES conversation(id) ON DELETE CASCADE,
    last_inquiry_at TIMESTAMPTZ NOT NULL,
    last_message_id UUID NOT NULL,
    PRIMARY KEY (person_id, offer_id)
);

-- Backfill only messages sent by a non-owner in the offer owner's conversation.
INSERT INTO person_inquired_offer_activity (
    person_id, offer_id, conversation_id, last_inquiry_at, last_message_id
)
SELECT DISTINCT ON (m.sender_id, m.offer_id)
       m.sender_id, m.offer_id, m.conversation_id, m.created_at, m.id
FROM message m
JOIN offer o ON o.id = m.offer_id
JOIN conversation c ON c.id = m.conversation_id
WHERE m.offer_id IS NOT NULL
  AND m.sender_id <> o.user_id
  AND c.user_low = LEAST(m.sender_id, o.user_id)
  AND c.user_high = GREATEST(m.sender_id, o.user_id)
ORDER BY m.sender_id, m.offer_id, m.created_at DESC, m.id DESC;

CREATE INDEX idx_person_inquired_offer_activity
    ON person_inquired_offer_activity
       (person_id, last_inquiry_at DESC, offer_id DESC);

CREATE FUNCTION maintain_person_inquired_offer_activity() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.offer_id IS NULL THEN
        RETURN NEW;
    END IF;

    INSERT INTO person_inquired_offer_activity (
        person_id, offer_id, conversation_id, last_inquiry_at, last_message_id
    )
    SELECT NEW.sender_id, o.id, c.id, NEW.created_at, NEW.id
    FROM offer o
    JOIN conversation c ON c.id = NEW.conversation_id
    WHERE o.id = NEW.offer_id
      AND NEW.sender_id <> o.user_id
      AND c.user_low = LEAST(NEW.sender_id, o.user_id)
      AND c.user_high = GREATEST(NEW.sender_id, o.user_id)
    ON CONFLICT (person_id, offer_id) DO UPDATE
    SET conversation_id = EXCLUDED.conversation_id,
        last_inquiry_at = EXCLUDED.last_inquiry_at,
        last_message_id = EXCLUDED.last_message_id
    WHERE (EXCLUDED.last_inquiry_at, EXCLUDED.last_message_id) >
          (person_inquired_offer_activity.last_inquiry_at,
           person_inquired_offer_activity.last_message_id);

    RETURN NEW;
END;
$$;

CREATE TRIGGER message_maintain_person_inquired_offer_activity
AFTER INSERT ON message
FOR EACH ROW
EXECUTE FUNCTION maintain_person_inquired_offer_activity();
