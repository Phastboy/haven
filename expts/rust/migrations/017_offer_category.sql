-- Prelaunch vocabulary: existing rows are test/seed data, not live offers.
ALTER TABLE offer ADD COLUMN category TEXT NOT NULL DEFAULT 'other';
ALTER TABLE offer ALTER COLUMN category DROP DEFAULT;
ALTER TABLE offer ADD CONSTRAINT check_offer_category CHECK (category IN (
    'fashion-beauty',
    'electronics',
    'home-furniture',
    'housing-real-estate',
    'coaching-consultation',
    'repairs-home-services',
    'tools-equipment',
    'books-learning',
    'food-groceries',
    'stays-accommodation',
    'other'
));
CREATE INDEX idx_offer_category_created
    ON offer (category, created_at DESC, id DESC)
    WHERE image_count >= 1;
