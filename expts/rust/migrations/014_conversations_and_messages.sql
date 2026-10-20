-- 014_conversations_and_messages.sql: Private 1:1 messaging and offer inquiries

CREATE TABLE conversation (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_low        UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    user_high       UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_message_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_conversation_pair_ordered CHECK (user_low < user_high),
    CONSTRAINT uq_conversation_pair UNIQUE (user_low, user_high)
);

CREATE TABLE conversation_participant (
    conversation_id UUID NOT NULL REFERENCES conversation(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    last_read_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (conversation_id, user_id)
);

CREATE INDEX idx_participant_user ON conversation_participant(user_id);

CREATE TABLE message (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversation(id) ON DELETE CASCADE,
    sender_id       UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    body            VARCHAR(2000) NOT NULL CHECK (length(btrim(body)) > 0),
    offer_id        UUID REFERENCES offer(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_message_thread ON message(conversation_id, created_at, id);
CREATE INDEX idx_message_offer ON message(offer_id, created_at DESC) WHERE offer_id IS NOT NULL;
