-- 012_images.sql: Image support for offers and user profiles

-- 1. Profile avatar on user table (nullable, max 120 chars for Cloudinary public_id)
ALTER TABLE "user"
    ADD COLUMN avatar_public_id VARCHAR(120);

ALTER TABLE "user"
    ADD CONSTRAINT check_user_avatar_public_id_length
    CHECK (avatar_public_id IS NULL OR (char_length(trim(avatar_public_id)) >= 1 AND char_length(avatar_public_id) <= 120));

-- 2. Offer images table (enforcing 1-5 images per offer and position order)
CREATE TABLE offer_image (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    offer_id UUID NOT NULL REFERENCES offer(id) ON DELETE CASCADE,
    public_id VARCHAR(120) NOT NULL CHECK (char_length(trim(public_id)) >= 1 AND char_length(public_id) <= 120),
    position SMALLINT NOT NULL CHECK (position >= 1 AND position <= 5),
    width INTEGER NOT NULL CHECK (width > 0),
    height INTEGER NOT NULL CHECK (height > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_offer_image_position UNIQUE (offer_id, position),
    CONSTRAINT uq_offer_image_public_id UNIQUE (public_id)
);

CREATE INDEX idx_offer_image_offer_pos ON offer_image (offer_id, position);

-- 3. Offer image count cache / visibility flag on offer table for O(1) feed queries
ALTER TABLE offer
    ADD COLUMN image_count SMALLINT NOT NULL DEFAULT 0 CHECK (image_count >= 0 AND image_count <= 5);

-- Index for public feed filtering (only show offers with image_count >= 1)
CREATE INDEX idx_offer_feed_visible ON offer (created_at DESC, id DESC)
    WHERE image_count >= 1;
