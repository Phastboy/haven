-- Read-only evidence for category vocabulary review; no synthetic-title filtering.
-- Use a verified real-inventory connection, never the synthetic load-test database.
-- Example (libpq service configured locally; no credentials in shell arguments):
-- PGSERVICE=haven-inventory psql -Xq -v ON_ERROR_STOP=1 \
--   -f bin/inventory_offer_titles.sql > /tmp/haven-offer-titles.csv
-- Keep this export local; it includes titles of offers without public photos.
-- Group topics manually and record counts, percentages and ambiguous examples
-- in V0.7_PLAN.md. Category/slugs must be accepted before schema migration.

BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;
SET LOCAL statement_timeout = '10s';
SET LOCAL lock_timeout = '1s';

COPY (
    WITH titles AS (
        SELECT title,
               COUNT(*) AS total_offers,
               COUNT(*) FILTER (WHERE image_count >= 1) AS visible_offers
        FROM public.offer
        GROUP BY title
    )
    SELECT title, total_offers, visible_offers,
           SUM(total_offers) OVER () AS inventory_total_offers,
           SUM(visible_offers) OVER () AS inventory_visible_offers
    FROM titles
    ORDER BY total_offers DESC, title
) TO STDOUT WITH (FORMAT CSV, HEADER TRUE);

COMMIT;
