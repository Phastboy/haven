SELECT o.id,o.title,m.created_at AS last_inquiry_at FROM message m
JOIN conversation c ON c.id=m.conversation_id JOIN offer o ON o.id=m.offer_id
WHERE m.sender_id='00000000-0000-0000-0000-000000000001'::uuid
AND o.image_count>=1 AND o.user_id<>m.sender_id
AND c.user_low=LEAST(m.sender_id,o.user_id) AND c.user_high=GREATEST(m.sender_id,o.user_id)
AND m.id=(SELECT newer.id FROM message newer WHERE newer.sender_id=m.sender_id
 AND newer.offer_id=m.offer_id AND newer.conversation_id=m.conversation_id
 ORDER BY newer.created_at DESC,newer.id DESC LIMIT 1)
ORDER BY m.created_at DESC,m.offer_id DESC LIMIT 21;
