-- Reproducible local planner experiment; all fixture writes are TEMP only.
-- Run: psql -X -v ON_ERROR_STOP=1 -d haven_rust -f V0.7_QUERY_PLANS.sql
\timing on
BEGIN;
SET LOCAL statement_timeout = '60s';
CREATE TEMP TABLE offer AS SELECT o.*, row_number() OVER (ORDER BY id) AS fixture_n FROM public.offer o;
CREATE INDEX ON offer(id);
CREATE INDEX ON offer(created_at DESC,id DESC) WHERE image_count >= 1;
CREATE TEMP TABLE conversation AS SELECT id, LEAST(user_id,'00000000-0000-0000-0000-000000000001'::uuid) user_low, GREATEST(user_id,'00000000-0000-0000-0000-000000000001'::uuid) user_high FROM offer WHERE fixture_n<=10000;
CREATE UNIQUE INDEX ON conversation(id);
CREATE TEMP TABLE message AS SELECT o.id offer_id, o.id conversation_id, '00000000-0000-0000-0000-000000000001'::uuid sender_id, o.created_at + r * interval '1 second' created_at FROM offer o CROSS JOIN generate_series(1,3) r WHERE fixture_n<=10000;
-- Sparse sender has five qualifying offers and a separate participant pair.
INSERT INTO conversation SELECT id, LEAST(user_id,'00000000-0000-0000-0000-000000000002'::uuid), GREATEST(user_id,'00000000-0000-0000-0000-000000000002'::uuid) FROM offer WHERE fixture_n BETWEEN 10001 AND 10005;
INSERT INTO message SELECT id,id,'00000000-0000-0000-0000-000000000002'::uuid,created_at FROM offer WHERE fixture_n BETWEEN 10001 AND 10005;
CREATE INDEX ON message(offer_id,created_at DESC) WHERE offer_id IS NOT NULL;
ANALYZE offer;
ANALYZE conversation;
ANALYZE message;
\echo baseline person=1 exists
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title FROM offer o WHERE o.image_count>=1 AND o.user_id<>'00000000-0000-0000-0000-000000000001'::uuid AND EXISTS (SELECT 1 FROM message m JOIN conversation c ON c.id=m.conversation_id WHERE m.sender_id='00000000-0000-0000-0000-000000000001'::uuid AND m.offer_id=o.id AND ((c.user_low='00000000-0000-0000-0000-000000000001'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000001'::uuid AND c.user_low=o.user_id))) ORDER BY o.created_at DESC,o.id DESC LIMIT 21;
\echo baseline person=1 activity
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title,h.last_inquiry_at FROM (SELECT m.offer_id,MAX(m.created_at) last_inquiry_at FROM message m JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id WHERE m.sender_id='00000000-0000-0000-0000-000000000001'::uuid AND o.user_id<>'00000000-0000-0000-0000-000000000001'::uuid AND ((c.user_low='00000000-0000-0000-0000-000000000001'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000001'::uuid AND c.user_low=o.user_id)) GROUP BY m.offer_id) h JOIN offer o ON o.id=h.offer_id WHERE o.image_count>=1 ORDER BY h.last_inquiry_at DESC,o.id DESC LIMIT 21;
\echo baseline person=2 exists
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title FROM offer o WHERE o.image_count>=1 AND o.user_id<>'00000000-0000-0000-0000-000000000002'::uuid AND EXISTS (SELECT 1 FROM message m JOIN conversation c ON c.id=m.conversation_id WHERE m.sender_id='00000000-0000-0000-0000-000000000002'::uuid AND m.offer_id=o.id AND ((c.user_low='00000000-0000-0000-0000-000000000002'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000002'::uuid AND c.user_low=o.user_id))) ORDER BY o.created_at DESC,o.id DESC LIMIT 21;
\echo baseline person=2 activity
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title,h.last_inquiry_at FROM (SELECT m.offer_id,MAX(m.created_at) last_inquiry_at FROM message m JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id WHERE m.sender_id='00000000-0000-0000-0000-000000000002'::uuid AND o.user_id<>'00000000-0000-0000-0000-000000000002'::uuid AND ((c.user_low='00000000-0000-0000-0000-000000000002'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000002'::uuid AND c.user_low=o.user_id)) GROUP BY m.offer_id) h JOIN offer o ON o.id=h.offer_id WHERE o.image_count>=1 ORDER BY h.last_inquiry_at DESC,o.id DESC LIMIT 21;
CREATE INDEX idx_fixture_sender_offer_time ON message(sender_id,offer_id,created_at DESC) WHERE offer_id IS NOT NULL;
ANALYZE message;
\echo indexed person=1 exists
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title FROM offer o WHERE o.image_count>=1 AND o.user_id<>'00000000-0000-0000-0000-000000000001'::uuid AND EXISTS (SELECT 1 FROM message m JOIN conversation c ON c.id=m.conversation_id WHERE m.sender_id='00000000-0000-0000-0000-000000000001'::uuid AND m.offer_id=o.id AND ((c.user_low='00000000-0000-0000-0000-000000000001'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000001'::uuid AND c.user_low=o.user_id))) ORDER BY o.created_at DESC,o.id DESC LIMIT 21;
\echo indexed person=1 activity
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title,h.last_inquiry_at FROM (SELECT m.offer_id,MAX(m.created_at) last_inquiry_at FROM message m JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id WHERE m.sender_id='00000000-0000-0000-0000-000000000001'::uuid AND o.user_id<>'00000000-0000-0000-0000-000000000001'::uuid AND ((c.user_low='00000000-0000-0000-0000-000000000001'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000001'::uuid AND c.user_low=o.user_id)) GROUP BY m.offer_id) h JOIN offer o ON o.id=h.offer_id WHERE o.image_count>=1 ORDER BY h.last_inquiry_at DESC,o.id DESC LIMIT 21;
\echo indexed person=2 exists
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title FROM offer o WHERE o.image_count>=1 AND o.user_id<>'00000000-0000-0000-0000-000000000002'::uuid AND EXISTS (SELECT 1 FROM message m JOIN conversation c ON c.id=m.conversation_id WHERE m.sender_id='00000000-0000-0000-0000-000000000002'::uuid AND m.offer_id=o.id AND ((c.user_low='00000000-0000-0000-0000-000000000002'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000002'::uuid AND c.user_low=o.user_id))) ORDER BY o.created_at DESC,o.id DESC LIMIT 21;
\echo indexed person=2 activity
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title,h.last_inquiry_at FROM (SELECT m.offer_id,MAX(m.created_at) last_inquiry_at FROM message m JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id WHERE m.sender_id='00000000-0000-0000-0000-000000000002'::uuid AND o.user_id<>'00000000-0000-0000-0000-000000000002'::uuid AND ((c.user_low='00000000-0000-0000-0000-000000000002'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000002'::uuid AND c.user_low=o.user_id)) GROUP BY m.offer_id) h JOIN offer o ON o.id=h.offer_id WHERE o.image_count>=1 ORDER BY h.last_inquiry_at DESC,o.id DESC LIMIT 21;

\echo indexed person=1 activity-single-offer-join
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title,MAX(m.created_at) last_inquiry_at
FROM message m JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id
WHERE m.sender_id='00000000-0000-0000-0000-000000000001'::uuid AND o.user_id<>'00000000-0000-0000-0000-000000000001'::uuid AND o.image_count>=1
AND ((c.user_low='00000000-0000-0000-0000-000000000001'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000001'::uuid AND c.user_low=o.user_id))
GROUP BY o.id,o.title ORDER BY MAX(m.created_at) DESC,o.id DESC LIMIT 21;
\echo indexed person=2 activity-single-offer-join
EXPLAIN (ANALYZE, BUFFERS)
SELECT o.id,o.title,MAX(m.created_at) last_inquiry_at
FROM message m JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id
WHERE m.sender_id='00000000-0000-0000-0000-000000000002'::uuid AND o.user_id<>'00000000-0000-0000-0000-000000000002'::uuid AND o.image_count>=1
AND ((c.user_low='00000000-0000-0000-0000-000000000002'::uuid AND c.user_high=o.user_id) OR (c.user_high='00000000-0000-0000-0000-000000000002'::uuid AND c.user_low=o.user_id))
GROUP BY o.id,o.title ORDER BY MAX(m.created_at) DESC,o.id DESC LIMIT 21;
ROLLBACK;
