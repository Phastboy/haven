-- 011_user_profile_fields.sql: Add profile fields to user entity for v0.3.x

-- 1. Add profile columns with temporary safe defaults for backward compatibility during rollout
ALTER TABLE "user"
    ADD COLUMN display_name VARCHAR(50) NOT NULL DEFAULT 'Member',
    ADD COLUMN handle VARCHAR(30) NOT NULL DEFAULT ('user-' || substr(gen_random_uuid()::text, 1, 8)),
    ADD COLUMN bio TEXT;

-- 2. Backfill existing user records using their account email local-part
-- Generates a clean URL-safe handle: alphanumeric with hyphens, falling back to 'user-<id>'
UPDATE "user" u
SET
    display_name = COALESCE(
        NULLIF(trim(left(split_part(a.email, '@', 1), 50)), ''),
        'Member'
    ),
    handle = COALESCE(
        NULLIF(
            left(
                regexp_replace(
                    regexp_replace(lower(split_part(a.email, '@', 1)), '[^a-z0-9]+', '-', 'g'),
                    '^-+|-+$', '', 'g'
                ),
                23
            ),
            ''
        ),
        'user'
    ) || '-' || substr(u.id::text, 1, 6)
FROM account a
WHERE u.account_id = a.id;

-- 3. Remove temporary default on handle and display_name so future inserts explicitly supply them
ALTER TABLE "user" ALTER COLUMN handle DROP DEFAULT;
ALTER TABLE "user" ALTER COLUMN display_name DROP DEFAULT;

-- 4. Enforce domain constraints on user profile fields
ALTER TABLE "user" ADD CONSTRAINT check_user_display_name_length
    CHECK (char_length(trim(display_name)) >= 2 AND char_length(display_name) <= 50);

ALTER TABLE "user" ADD CONSTRAINT check_user_handle_format
    CHECK (handle ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(handle) >= 3 AND char_length(handle) <= 30);

ALTER TABLE "user" ADD CONSTRAINT check_user_bio_length
    CHECK (bio IS NULL OR char_length(bio) <= 500);

-- 5. Create unique index on lowercase handle for fast case-insensitive profile lookups (/profiles/{handle})
CREATE UNIQUE INDEX idx_user_handle ON "user" (lower(handle));
