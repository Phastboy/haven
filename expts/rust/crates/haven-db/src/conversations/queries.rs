//! Database queries for conversations and messages.

#![allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::as_conversions,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::arithmetic_side_effects,
    reason = "Database query mapping functions"
)]

use chrono::Utc;
use haven_domain::conversation::{
    Conversation, ConversationCursor, ConversationId, ConversationInboxPage, ConversationPair,
    Message, MessageCursor, MessagePage, NewMessage, OfferInquirer, OfferInquirersPage,
};
use haven_domain::offer::OfferId;
use haven_domain::ports::RepoError;
use haven_domain::user::UserId;
use uuid::Uuid;

mod unread;
pub use unread::{mark_read, unread_exists};

use super::rows::{ConversationRow, InboxRow, MessageRow};
use crate::DbPool;

pub async fn get_or_create(
    pool: &DbPool,
    pair: ConversationPair,
) -> Result<Conversation, RepoError> {
    let low = pair.low().as_uuid();
    let high = pair.high().as_uuid();

    let mut tx = pool.begin().await.map_err(crate::map_sqlx_err)?;

    // Insert conversation if not exists
    let maybe_created = sqlx::query_as!(
        ConversationRow,
        r#"
        INSERT INTO conversation (user_low, user_high)
        VALUES ($1, $2)
        ON CONFLICT (user_low, user_high) DO NOTHING
        RETURNING id, user_low, user_high, created_at, last_message_at
        "#,
        low,
        high
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::map_sqlx_err)?;

    let row = match maybe_created {
        Some(r) => {
            // New conversation: insert participants
            sqlx::query!(
                r#"
                INSERT INTO conversation_participant (conversation_id, user_id, last_read_at)
                VALUES ($1, $2, $3), ($1, $4, $3)
                ON CONFLICT (conversation_id, user_id) DO NOTHING
                "#,
                r.id,
                low,
                r.created_at,
                high
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::map_sqlx_err)?;

            r
        }
        None => {
            // Already existed: select it
            sqlx::query_as!(
                ConversationRow,
                r#"
                SELECT id, user_low, user_high, created_at, last_message_at
                FROM conversation
                WHERE user_low = $1 AND user_high = $2
                "#,
                low,
                high
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(crate::map_sqlx_err)?
        }
    };

    tx.commit().await.map_err(crate::map_sqlx_err)?;

    row.into_domain()
}

pub async fn find_by_id(
    pool: &DbPool,
    id: ConversationId,
) -> Result<Option<Conversation>, RepoError> {
    let conv_id = id.as_uuid();

    let row = sqlx::query_as!(
        ConversationRow,
        r#"
        SELECT id, user_low, user_high, created_at, last_message_at
        FROM conversation
        WHERE id = $1
        "#,
        conv_id
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    match row {
        Some(r) => Ok(Some(r.into_domain()?)),
        None => Ok(None),
    }
}

pub async fn append(
    pool: &DbPool,
    conversation_id: ConversationId,
    new_message: NewMessage,
) -> Result<Message, RepoError> {
    let c_id = conversation_id.as_uuid();
    let s_id = new_message.sender_id.as_uuid();
    let off_id = new_message.offer_id.map(|o| o.as_uuid());
    let body_str = new_message.body.as_str();

    let mut tx = pool.begin().await.map_err(crate::map_sqlx_err)?;

    // Serialize sends and reads before assigning a message timestamp.
    unread::lock_participant(&mut tx, conversation_id, new_message.sender_id).await?;

    // 2. Insert message
    let msg_row = sqlx::query_as!(
        MessageRow,
        r#"
        INSERT INTO message (conversation_id, sender_id, body, offer_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, conversation_id, sender_id, body, offer_id, created_at
        "#,
        c_id,
        s_id,
        body_str,
        off_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::map_sqlx_err)?;

    // 3. Update conversation last_message_at
    sqlx::query!(
        r#"
        UPDATE conversation
        SET last_message_at = $1
        WHERE id = $2
        "#,
        msg_row.created_at,
        c_id
    )
    .execute(&mut *tx)
    .await
    .map_err(crate::map_sqlx_err)?;

    // 4. Update sender participant last_read_at
    sqlx::query!(
        r#"
        INSERT INTO conversation_participant (conversation_id, user_id, last_read_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (conversation_id, user_id)
        DO UPDATE SET last_read_at = EXCLUDED.last_read_at
        "#,
        c_id,
        s_id,
        msg_row.created_at
    )
    .execute(&mut *tx)
    .await
    .map_err(crate::map_sqlx_err)?;

    tx.commit().await.map_err(crate::map_sqlx_err)?;

    msg_row.into_domain()
}

pub async fn list_for_user(
    pool: &DbPool,
    user_id: UserId,
    cursor: Option<&str>,
    limit: usize,
) -> Result<ConversationInboxPage, RepoError> {
    let u_id = user_id.as_uuid();
    let parsed_cursor = match cursor {
        Some(c) => ConversationCursor::decode(c).ok(),
        None => None,
    };

    let fetch_limit = (limit.clamp(1, 100) + 1) as i64;

    let query = match parsed_cursor {
        Some(cur) => {
            let cursor_time = cur.last_message_at;
            let cursor_id = cur.id.as_uuid();
            sqlx::query_as!(
                InboxRow,
                r#"
                SELECT 
                    c.id AS conversation_id,
                    (CASE WHEN c.user_low = $1 THEN c.user_high ELSE c.user_low END) AS "other_user_id!",
                    u.display_name AS other_display_name,
                    u.handle AS other_handle,
                    u.avatar_public_id AS other_avatar_public_id,
                    (SELECT m.body FROM message m WHERE m.conversation_id = c.id ORDER BY m.created_at DESC, m.id DESC LIMIT 1) AS last_message_body,
                    c.last_message_at,
                    (
                        cp.last_read_at < c.last_message_at
                        AND EXISTS (
                            SELECT 1 FROM message lm 
                            WHERE lm.conversation_id = c.id 
                            ORDER BY lm.created_at DESC, lm.id DESC 
                            LIMIT 1
                        )
                        AND (
                            SELECT lm.sender_id FROM message lm 
                            WHERE lm.conversation_id = c.id 
                            ORDER BY lm.created_at DESC, lm.id DESC 
                            LIMIT 1
                        ) != $1
                    ) AS "is_unread!"
                FROM conversation_participant cp
                JOIN conversation c ON c.id = cp.conversation_id
                JOIN "user" u ON u.id = CASE WHEN c.user_low = $1 THEN c.user_high ELSE c.user_low END
                WHERE cp.user_id = $1
                  AND (c.last_message_at < $2 OR (c.last_message_at = $2 AND c.id < $3))
                ORDER BY c.last_message_at DESC, c.id DESC
                LIMIT $4
                "#,
                u_id,
                cursor_time,
                cursor_id,
                fetch_limit
            )
            .fetch_all(pool)
            .await
            .map_err(crate::map_sqlx_err)?
        }
        None => {
            sqlx::query_as!(
                InboxRow,
                r#"
                SELECT 
                    c.id AS conversation_id,
                    (CASE WHEN c.user_low = $1 THEN c.user_high ELSE c.user_low END) AS "other_user_id!",
                    u.display_name AS other_display_name,
                    u.handle AS other_handle,
                    u.avatar_public_id AS other_avatar_public_id,
                    (SELECT m.body FROM message m WHERE m.conversation_id = c.id ORDER BY m.created_at DESC, m.id DESC LIMIT 1) AS last_message_body,
                    c.last_message_at,
                    (
                        cp.last_read_at < c.last_message_at
                        AND EXISTS (
                            SELECT 1 FROM message lm 
                            WHERE lm.conversation_id = c.id 
                            ORDER BY lm.created_at DESC, lm.id DESC 
                            LIMIT 1
                        )
                        AND (
                            SELECT lm.sender_id FROM message lm 
                            WHERE lm.conversation_id = c.id 
                            ORDER BY lm.created_at DESC, lm.id DESC 
                            LIMIT 1
                        ) != $1
                    ) AS "is_unread!"
                FROM conversation_participant cp
                JOIN conversation c ON c.id = cp.conversation_id
                JOIN "user" u ON u.id = CASE WHEN c.user_low = $1 THEN c.user_high ELSE c.user_low END
                WHERE cp.user_id = $1
                ORDER BY c.last_message_at DESC, c.id DESC
                LIMIT $2
                "#,
                u_id,
                fetch_limit
            )
            .fetch_all(pool)
            .await
            .map_err(crate::map_sqlx_err)?
        }
    };

    let has_more = query.len() > limit;
    let rows: Vec<InboxRow> = query.into_iter().take(limit).collect();

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(row.into_domain()?);
    }

    let next_cursor = if has_more {
        items
            .last()
            .map(|it| ConversationCursor::new(it.last_message_at, it.id).encode())
    } else {
        None
    };

    Ok(ConversationInboxPage { items, next_cursor })
}

pub async fn messages(
    pool: &DbPool,
    conversation_id: ConversationId,
    cursor: Option<&str>,
    limit: usize,
) -> Result<MessagePage, RepoError> {
    let c_id = conversation_id.as_uuid();
    let parsed_cursor = match cursor {
        Some(c) => MessageCursor::decode(c).ok(),
        None => None,
    };

    let fetch_limit = (limit.clamp(1, 100) + 1) as i64;

    let query = match parsed_cursor {
        Some(cur) => {
            let cursor_time = cur.created_at;
            let cursor_id = cur.id.as_uuid();
            sqlx::query_as!(
                MessageRow,
                r#"
                SELECT id, conversation_id, sender_id, body, offer_id, created_at
                FROM message
                WHERE conversation_id = $1
                  AND (created_at < $2 OR (created_at = $2 AND id < $3))
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#,
                c_id,
                cursor_time,
                cursor_id,
                fetch_limit
            )
            .fetch_all(pool)
            .await
            .map_err(crate::map_sqlx_err)?
        }
        None => sqlx::query_as!(
            MessageRow,
            r#"
                SELECT id, conversation_id, sender_id, body, offer_id, created_at
                FROM message
                WHERE conversation_id = $1
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#,
            c_id,
            fetch_limit
        )
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?,
    };

    let has_more = query.len() > limit;
    let mut rows: Vec<MessageRow> = query.into_iter().take(limit).collect();
    rows.reverse();

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(row.into_domain()?);
    }

    let next_cursor = if has_more {
        items
            .first()
            .map(|it| MessageCursor::new(it.created_at, it.id).encode())
    } else {
        None
    };

    Ok(MessagePage { items, next_cursor })
}

struct InquirerRow {
    sender_id: Uuid,
    conversation_id: Uuid,
    last_inquiry_at: chrono::DateTime<Utc>,
    message_count: Option<i64>,
}

pub async fn list_offer_inquirers(
    pool: &DbPool,
    offer_id: OfferId,
    _cursor: Option<&str>,
    limit: usize,
) -> Result<OfferInquirersPage, RepoError> {
    let o_id = offer_id.as_uuid();
    let fetch_limit = limit.clamp(1, 100) as i64;

    let rows = sqlx::query_as!(
        InquirerRow,
        r#"
        SELECT 
            m.sender_id,
            m.conversation_id,
            MAX(m.created_at) AS "last_inquiry_at!",
            COUNT(m.id) AS message_count
        FROM message m
        JOIN offer o ON o.id = m.offer_id
        WHERE m.offer_id = $1
          AND m.sender_id <> o.user_id
        GROUP BY m.sender_id, m.conversation_id
        ORDER BY MAX(m.created_at) DESC
        LIMIT $2
        "#,
        o_id,
        fetch_limit
    )
    .fetch_all(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    let items = rows
        .into_iter()
        .map(|r| OfferInquirer {
            user_id: UserId::from_uuid(r.sender_id),
            conversation_id: ConversationId::from_uuid(r.conversation_id),
            last_inquiry_at: r.last_inquiry_at,
            message_count: r.message_count.unwrap_or(0) as u64,
        })
        .collect();

    Ok(OfferInquirersPage {
        items,
        next_cursor: None,
    })
}
