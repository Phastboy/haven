CREATE TABLE saved_offer (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    offer_id UUID NOT NULL REFERENCES offer(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, id),
    UNIQUE (user_id, offer_id)
);
CREATE INDEX idx_saved_offer_user_created
    ON saved_offer (user_id, created_at DESC, id DESC);

CREATE TABLE follow (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    followed_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, id),
    UNIQUE (user_id, followed_id),
    CONSTRAINT follow_not_self CHECK (user_id <> followed_id)
);
CREATE INDEX idx_follow_follower_created
    ON follow (user_id, created_at DESC, id DESC);
