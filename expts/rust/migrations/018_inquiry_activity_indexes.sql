-- Latest qualifying message per offer, streamed in personal activity order.
-- Conversation checks remain in the query; an arbitrary offer tag is not an inquiry.
CREATE INDEX idx_message_sender_activity
    ON message (sender_id, created_at DESC, offer_id DESC)
    INCLUDE (id, conversation_id) WHERE offer_id IS NOT NULL;
CREATE INDEX idx_message_sender_offer_latest
    ON message (sender_id, offer_id, conversation_id, created_at DESC, id DESC)
    WHERE offer_id IS NOT NULL;
