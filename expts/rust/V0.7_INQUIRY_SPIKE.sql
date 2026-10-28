-- Query used against an isolated synthetic schema; set search_path before execution.
SELECT o.id,o.title,h.last_inquiry_at FROM (
 SELECT DISTINCT ON (offer_id,conversation_id) offer_id,conversation_id,created_at AS last_inquiry_at
 FROM message WHERE sender_id='00000000-0000-0000-0000-000000000001'::uuid AND offer_id IS NOT NULL
 ORDER BY offer_id,conversation_id,created_at DESC
) h JOIN conversation c ON c.id=h.conversation_id JOIN offer o ON o.id=h.offer_id
WHERE o.image_count>=1 AND o.user_id<>'00000000-0000-0000-0000-000000000001'::uuid
AND c.user_low=LEAST('00000000-0000-0000-0000-000000000001'::uuid,o.user_id) AND c.user_high=GREATEST('00000000-0000-0000-0000-000000000001'::uuid,o.user_id)
ORDER BY h.last_inquiry_at DESC,o.id DESC LIMIT 21;
