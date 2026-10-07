use crate::DbPool;
use async_trait::async_trait;
use haven_domain::offer::{CreateOffer, CurrencyCode, Offer, OfferId, Price, UpdateOffer, UserId};
use haven_domain::ports::{IdempotencyKey, OfferRepository, RepoError};
use uuid::Uuid;

pub struct PostgresOfferRepository {
    pub pool: DbPool,
}

impl PostgresOfferRepository {
    #[allow(clippy::too_many_arguments, reason = "Internal DB mapping function")]
    fn map_row(
        id: Uuid,
        user_id: Uuid,
        title: String,
        description: Option<String>,
        price: i32,
        currency: &str,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Offer, RepoError> {
        let p = Price::new(price).map_err(|_| RepoError::Corrupt("Invalid price".into()))?;
        let c = CurrencyCode::parse(currency)
            .map_err(|_| RepoError::Corrupt("Invalid currency".into()))?;

        Ok(Offer {
            id: OfferId(id),
            user_id: UserId(user_id),
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
            SELECT id, user_id, title, description, price, currency, created_at, updated_at
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
            SELECT id, user_id, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE user_id = $1
            ORDER BY created_at DESC
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

        let row = sqlx::query!(
            r#"
            INSERT INTO offer (user_id, title, description, price, currency, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, idempotency_key) DO NOTHING
            RETURNING id, user_id, title, description, price, currency, created_at, updated_at
            "#,
            u_id,
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
                SELECT id, user_id, title, description, price, currency, created_at, updated_at
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
        let price_val = update_offer.price.map(|p| p.as_i32());
        let currency_val = update_offer.currency.as_ref().map(CurrencyCode::as_str);

        let row = sqlx::query!(
            r#"
            UPDATE offer
            SET title = $1, description = $2, price = COALESCE($3, price), currency = COALESCE($4, currency), updated_at = NOW()
            WHERE id = $5 AND user_id = $6
            RETURNING id, user_id, title, description, price, currency, created_at, updated_at
            "#,
            update_offer.title,
            update_offer.description,
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
        sqlx::query!(
            "INSERT INTO account (id, email) VALUES ($1, $2)",
            account_id,
            format!("{}@example.com", account_id)
        )
        .execute(&pool)
        .await
        .unwrap();

        let user_id = UserId(Uuid::new_v4());
        sqlx::query!(
            "INSERT INTO \"user\" (id, account_id) VALUES ($1, $2)",
            user_id.as_uuid(),
            account_id
        )
        .execute(&pool)
        .await
        .unwrap();

        // 2. Create an offer
        let co = CreateOffer {
            title: "Original Title".to_string(),
            description: None,
            price: Price::new(100).unwrap(),
            currency: CurrencyCode::parse("USD").unwrap(),
        };
        let offer = repo
            .create(user_id, IdempotencyKey(Uuid::new_v4()), &co)
            .await
            .unwrap();

        assert_eq!(offer.price.as_i32(), 100);
        assert_eq!(offer.currency.as_str(), "USD");

        // 3. Update title, leave price and currency unchanged (None)
        let uo1 = UpdateOffer {
            title: "New Title".to_string(),
            description: None,
            price: None,
            currency: None,
        };
        let offer1 = repo.update(offer.id, user_id, &uo1).await.unwrap();
        assert_eq!(offer1.title, "New Title");
        assert_eq!(offer1.price.as_i32(), 100); // Unchanged
        assert_eq!(offer1.currency.as_str(), "USD"); // Unchanged

        // 4. Update price only
        let uo2 = UpdateOffer {
            title: "New Title".to_string(),
            description: None,
            price: Some(Price::new(200).unwrap()),
            currency: None,
        };
        let offer2 = repo.update(offer.id, user_id, &uo2).await.unwrap();
        assert_eq!(offer2.price.as_i32(), 200);
        assert_eq!(offer2.currency.as_str(), "USD"); // Unchanged

        // 5. Update currency only
        let uo3 = UpdateOffer {
            title: "New Title".to_string(),
            description: None,
            price: None,
            currency: Some(CurrencyCode::parse("NGN").unwrap()),
        };
        let offer3 = repo.update(offer.id, user_id, &uo3).await.unwrap();
        assert_eq!(offer3.price.as_i32(), 200); // Unchanged
        assert_eq!(offer3.currency.as_str(), "NGN");
    }
}
