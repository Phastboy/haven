use crate::DbPool;
use async_trait::async_trait;
use haven_domain::offer::{
    CreateOffer, CurrencyCode, Offer, OfferCursor, OfferId, OfferPage, OfferSlug, Price,
    UpdateOffer, UserId,
};
use haven_domain::ports::{IdempotencyKey, OfferRepository, RepoError};
use uuid::Uuid;

pub struct PostgresOfferRepository {
    pub pool: DbPool,
}

impl PostgresOfferRepository {
    #[allow(
        clippy::too_many_arguments,
        clippy::needless_pass_by_value,
        reason = "Internal DB mapping function"
    )]
    fn map_row(
        id: Uuid,
        user_id: Uuid,
        slug: String,
        title: String,
        description: String,
        price: i32,
        currency: &str,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Offer, RepoError> {
        let p = Price::new(price).map_err(|_| RepoError::Corrupt("Invalid price".into()))?;
        let c = CurrencyCode::parse(currency)
            .map_err(|_| RepoError::Corrupt("Invalid currency".into()))?;
        let s = OfferSlug::parse(&slug).map_err(|_| RepoError::Corrupt("Invalid slug".into()))?;

        Ok(Offer {
            id: OfferId(id),
            user_id: UserId(user_id),
            slug: s,
            title,
            description,
            price: p,
            currency: c,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl OfferRepository for PostgresOfferRepository {
    async fn find_owned(
        &self,
        offer_id: OfferId,
        user_id: UserId,
    ) -> Result<Option<Offer>, RepoError> {
        let o_id = offer_id.as_uuid();
        let u_id = user_id.as_uuid();
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE id = $1 AND user_id = $2
            "#,
            o_id,
            u_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.created_at,
                r.updated_at,
            )
        })
        .transpose()
    }

    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Offer>, RepoError> {
        let u_id = user_id.as_uuid();
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE user_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
            u_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        let mut offers = Vec::with_capacity(rows.len());
        for r in rows {
            offers.push(Self::map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.created_at,
                r.updated_at,
            )?);
        }
        Ok(offers)
    }

    async fn find_by_slug(&self, slug: &OfferSlug) -> Result<Option<Offer>, RepoError> {
        let slug_str = slug.as_str();
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE slug = $1
            "#,
            slug_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.created_at,
                r.updated_at,
            )
        })
        .transpose()
    }

    async fn list_public(
        &self,
        cursor: Option<&OfferCursor>,
        limit: usize,
    ) -> Result<OfferPage, RepoError> {
        let capped_limit = limit.clamp(1, 50);
        let fetch_limit = match i64::try_from(capped_limit) {
            Ok(l) => l.saturating_add(1),
            Err(_) => 21,
        };

        let mut rows: Vec<Offer> = if let Some(c) = cursor {
            let c_time = c.created_at;
            let c_id = c.id.as_uuid();
            let rs = sqlx::query!(
                r#"
                SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
                FROM offer
                WHERE (created_at, id) < ($1, $2)
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#,
                c_time,
                c_id,
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(crate::map_sqlx_err)?;

            let mut items = Vec::with_capacity(rs.len());
            for r in rs {
                items.push(Self::map_row(
                    r.id,
                    r.user_id,
                    r.slug,
                    r.title,
                    r.description,
                    r.price,
                    &r.currency,
                    r.created_at,
                    r.updated_at,
                )?);
            }
            items
        } else {
            let rs = sqlx::query!(
                r#"
                SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
                FROM offer
                ORDER BY created_at DESC, id DESC
                LIMIT $1
                "#,
                fetch_limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(crate::map_sqlx_err)?;

            let mut items = Vec::with_capacity(rs.len());
            for r in rs {
                items.push(Self::map_row(
                    r.id,
                    r.user_id,
                    r.slug,
                    r.title,
                    r.description,
                    r.price,
                    &r.currency,
                    r.created_at,
                    r.updated_at,
                )?);
            }
            items
        };

        let has_more = rows.len() > capped_limit;
        if has_more {
            rows.truncate(capped_limit);
        }

        let next_cursor = if has_more {
            rows.last()
                .map(|item| OfferCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };

        Ok(OfferPage {
            items: rows,
            next_cursor,
        })
    }

    async fn create(
        &self,
        user_id: UserId,
        key: IdempotencyKey,
        create_offer: &CreateOffer,
    ) -> Result<Offer, RepoError> {
        let u_id = user_id.as_uuid();
        let price_val = create_offer.price.as_i32();
        let currency_val = create_offer.currency.as_str();
        let k = key.0;

        let offer_id = Uuid::new_v4();
        let mut suffix = String::new();
        for ch in offer_id.to_string().chars().take(8) {
            suffix.push(ch);
        }
        let slug = OfferSlug::from_title_and_suffix(&create_offer.title, &suffix)
            .map_err(|e| RepoError::Corrupt(e.to_string()))?;
        let slug_str = slug.as_str();

        let row = sqlx::query!(
            r#"
            INSERT INTO offer (id, user_id, slug, title, description, price, currency, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, idempotency_key) DO NOTHING
            RETURNING id, user_id, slug, title, description, price, currency, created_at, updated_at
            "#,
            offer_id,
            u_id,
            slug_str,
            create_offer.title,
            create_offer.description,
            price_val,
            currency_val,
            k
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        if let Some(r) = row {
            Self::map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.created_at,
                r.updated_at,
            )
        } else {
            let r = sqlx::query!(
                r#"
                SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
                FROM offer
                WHERE user_id = $1 AND idempotency_key = $2
                "#,
                u_id,
                k
            )
            .fetch_one(&self.pool)
            .await
            .map_err(crate::map_sqlx_err)?;

            Self::map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.created_at,
                r.updated_at,
            )
        }
    }

    async fn update(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        update_offer: &UpdateOffer,
    ) -> Result<Offer, RepoError> {
        let o_id = offer_id.as_uuid();
        let u_id = user_id.as_uuid();
        let title_val = update_offer.title.as_deref();
        let desc_val = update_offer.description.as_deref();
        let price_val = update_offer.price.map(|p| p.as_i32());
        let currency_val = update_offer.currency.as_ref().map(CurrencyCode::as_str);

        let row = sqlx::query!(
            r#"
            UPDATE offer
            SET title = COALESCE($1, title),
                description = COALESCE($2, description),
                price = COALESCE($3, price),
                currency = COALESCE($4, currency),
                updated_at = NOW()
            WHERE id = $5 AND user_id = $6
            RETURNING id, user_id, slug, title, description, price, currency, created_at, updated_at
            "#,
            title_val,
            desc_val,
            price_val,
            currency_val,
            o_id,
            u_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        match row {
            Some(r) => Self::map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.created_at,
                r.updated_at,
            ),
            None => Err(RepoError::NotFound),
        }
    }

    async fn delete(&self, offer_id: OfferId, user_id: UserId) -> Result<(), RepoError> {
        let o_id = offer_id.as_uuid();
        let u_id = user_id.as_uuid();
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM offer
            WHERE id = $1 AND user_id = $2
            "#,
            o_id,
            u_id
        )
        .execute(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?
        .rows_affected();

        if rows_affected == 0 {
            Err(RepoError::NotFound)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::manual_let_else,
    reason = "test assertions and optional local db connectivity"
)]
mod tests {
    use super::*;
    use haven_domain::offer::{CreateOffer, CurrencyCode, Price, UpdateOffer};
    use haven_domain::ports::{IdempotencyKey, OfferRepository};
    use sqlx::PgPool;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_update_patch_semantics() {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:password@127.0.0.1:5433/haven_rust".to_string());

        let pool = match PgPool::connect(&db_url).await {
            Ok(p) => p,
            Err(_) => return, // Skip if DB is not available
        };

        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

        let repo = PostgresOfferRepository { pool: pool.clone() };

        // 1. Create a dummy account and user
        let account_id = Uuid::new_v4();
        let email = format!("{account_id}@example.com");
        sqlx::query("INSERT INTO account (id, email) VALUES ($1, $2)")
            .bind(account_id)
            .bind(&email)
            .execute(&pool)
            .await
            .unwrap();

        let user_id = UserId(Uuid::new_v4());
        sqlx::query("INSERT INTO \"user\" (id, account_id) VALUES ($1, $2)")
            .bind(user_id.as_uuid())
            .bind(account_id)
            .execute(&pool)
            .await
            .unwrap();

        // 2. Create an offer
        let co = CreateOffer {
            title: "Original Title".to_string(),
            description: "Original Description".to_string(),
            price: Price::new(100).unwrap(),
            currency: CurrencyCode::parse("USD").unwrap(),
        };
        let offer = repo
            .create(user_id, IdempotencyKey(Uuid::new_v4()), &co)
            .await
            .unwrap();

        assert_eq!(offer.price.as_i32(), 100);
        assert_eq!(offer.currency.as_str(), "USD");
        assert_eq!(offer.description, "Original Description");

        // 3. Update title, leave description, price and currency unchanged (None)
        let uo1 = UpdateOffer {
            title: Some("New Title".to_string()),
            description: None,
            price: None,
            currency: None,
        };
        let offer1 = repo.update(offer.id, user_id, &uo1).await.unwrap();
        assert_eq!(offer1.title, "New Title");
        assert_eq!(offer1.description, "Original Description"); // Unchanged
        assert_eq!(offer1.price.as_i32(), 100); // Unchanged
        assert_eq!(offer1.currency.as_str(), "USD"); // Unchanged

        // 4. Update price only
        let uo2 = UpdateOffer {
            title: None,
            description: None,
            price: Some(Price::new(200).unwrap()),
            currency: None,
        };
        let offer2 = repo.update(offer.id, user_id, &uo2).await.unwrap();
        assert_eq!(offer2.title, "New Title"); // Unchanged
        assert_eq!(offer2.description, "Original Description"); // Unchanged
        assert_eq!(offer2.price.as_i32(), 200);
        assert_eq!(offer2.currency.as_str(), "USD"); // Unchanged

        // 5. Update currency and description
        let uo3 = UpdateOffer {
            title: None,
            description: Some("Updated Description".to_string()),
            price: None,
            currency: Some(CurrencyCode::parse("NGN").unwrap()),
        };
        let offer3 = repo.update(offer.id, user_id, &uo3).await.unwrap();
        assert_eq!(offer3.title, "New Title"); // Unchanged
        assert_eq!(offer3.description, "Updated Description");
        assert_eq!(offer3.price.as_i32(), 200); // Unchanged
        assert_eq!(offer3.currency.as_str(), "NGN");

        // 6. Test find_by_slug
        let found_by_slug = repo.find_by_slug(&offer.slug).await.unwrap();
        assert!(found_by_slug.is_some());
        let found = found_by_slug.unwrap();
        assert_eq!(found.id, offer.id);
        assert_eq!(found.slug, offer.slug);

        // 7. Seed multiple offers and test keyset feed pagination
        for i in 1..=5 {
            let req = CreateOffer {
                title: format!("Feed Offer {i}"),
                description: format!("Description for feed offer number {i}"),
                price: Price::new(10_i32.saturating_mul(i)).unwrap(),
                currency: CurrencyCode::parse("USD").unwrap(),
            };
            repo.create(user_id, IdempotencyKey(Uuid::new_v4()), &req)
                .await
                .unwrap();
        }

        // Fetch first page of 3 items
        let page1 = repo.list_public(None, 3).await.unwrap();
        assert_eq!(page1.items.len(), 3);
        assert!(page1.next_cursor.is_some());

        // Fetch second page using cursor
        let cursor_str = page1.next_cursor.unwrap();
        let cursor = haven_domain::offer::OfferCursor::decode(&cursor_str).unwrap();
        let page2 = repo.list_public(Some(&cursor), 3).await.unwrap();
        assert!(!page2.items.is_empty());

        // Ensure no overlap between page 1 and page 2
        let p1_ids: std::collections::HashSet<_> = page1.items.iter().map(|o| o.id).collect();
        for item in &page2.items {
            assert!(
                !p1_ids.contains(&item.id),
                "Keyset pagination must not duplicate items"
            );
        }
    }
}
