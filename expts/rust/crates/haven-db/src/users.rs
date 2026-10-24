use crate::DbPool;
use async_trait::async_trait;
use haven_domain::account::AccountId;
use haven_domain::offer::UserId;
use haven_domain::ports::{RepoError, UserRepository};
use haven_domain::session::HashedToken;
use haven_domain::user::{Bio, DisplayName, ProfileHandle, User, UserProfileUpdate};
use uuid::Uuid;

pub(crate) struct PostgresUserRepository {
    pool: DbPool,
}

pub(crate) struct UserDbRow {
    pub(crate) id: Uuid,
    pub(crate) account_id: Uuid,
    pub(crate) handle: String,
    pub(crate) display_name: String,
    pub(crate) bio: Option<String>,
    pub(crate) avatar_public_id: Option<String>,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,
}

impl PostgresUserRepository {
    pub(crate) fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub(crate) fn map_row(row: UserDbRow) -> Result<User, RepoError> {
        let handle = ProfileHandle::parse(&row.handle)
            .map_err(|_| RepoError::Corrupt("Invalid handle in database".into()))?;
        let display_name = DisplayName::parse(&row.display_name)
            .map_err(|_| RepoError::Corrupt("Invalid display name in database".into()))?;
        let bio = match row.bio {
            Some(b) if !b.trim().is_empty() => Some(
                Bio::parse(&b).map_err(|_| RepoError::Corrupt("Invalid bio in database".into()))?,
            ),
            _ => None,
        };
        let avatar_public_id = match row.avatar_public_id {
            Some(a) if !a.trim().is_empty() => {
                Some(haven_domain::image::ImagePublicId::parse(&a).map_err(|_| {
                    RepoError::Corrupt("Invalid avatar_public_id in database".into())
                })?)
            }
            _ => None,
        };

        Ok(User {
            id: UserId::from_uuid(row.id),
            account_id: AccountId::from_uuid(row.account_id),
            handle,
            display_name,
            bio,
            avatar_public_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_account(&self, account_id: AccountId) -> Result<Option<User>, RepoError> {
        let id_uuid = account_id.as_uuid();
        let row = sqlx::query!(
            r#"
            SELECT id, account_id, handle, display_name, bio, avatar_public_id, created_at, updated_at
            FROM "user"
            WHERE account_id = $1
            "#,
            id_uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(UserDbRow {
                id: r.id,
                account_id: r.account_id,
                handle: r.handle,
                display_name: r.display_name,
                bio: r.bio,
                avatar_public_id: r.avatar_public_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
        })
        .transpose()
    }

    async fn find_by_id(&self, user_id: UserId) -> Result<Option<User>, RepoError> {
        let id_uuid = user_id.as_uuid();
        let row = sqlx::query!(
            r#"
            SELECT id, account_id, handle, display_name, bio, avatar_public_id, created_at, updated_at
            FROM "user"
            WHERE id = $1
            "#,
            id_uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(UserDbRow {
                id: r.id,
                account_id: r.account_id,
                handle: r.handle,
                display_name: r.display_name,
                bio: r.bio,
                avatar_public_id: r.avatar_public_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
        })
        .transpose()
    }

    async fn find_by_handle(&self, handle: &ProfileHandle) -> Result<Option<User>, RepoError> {
        let handle_str = handle.as_str();
        let row = sqlx::query!(
            r#"
            SELECT id, account_id, handle, display_name, bio, avatar_public_id, created_at, updated_at
            FROM "user"
            WHERE lower(handle) = lower($1)
            "#,
            handle_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(UserDbRow {
                id: r.id,
                account_id: r.account_id,
                handle: r.handle,
                display_name: r.display_name,
                bio: r.bio,
                avatar_public_id: r.avatar_public_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
        })
        .transpose()
    }

    async fn create(
        &self,
        account_id: AccountId,
        profile: &UserProfileUpdate,
    ) -> Result<User, RepoError> {
        let id_uuid = account_id.as_uuid();
        let handle_str = profile.handle.as_str();
        let name_str = profile.display_name.as_str();
        let bio_str = profile.bio.as_ref().map(Bio::as_str);

        let row = sqlx::query!(
            r#"
            INSERT INTO "user" (account_id, handle, display_name, bio)
            VALUES ($1, $2, $3, $4)
            RETURNING id, account_id, handle, display_name, bio, avatar_public_id, created_at, updated_at
            "#,
            id_uuid,
            handle_str,
            name_str,
            bio_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        Self::map_row(UserDbRow {
            id: row.id,
            account_id: row.account_id,
            handle: row.handle,
            display_name: row.display_name,
            bio: row.bio,
            avatar_public_id: row.avatar_public_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update_profile(
        &self,
        user_id: UserId,
        profile: &UserProfileUpdate,
    ) -> Result<User, RepoError> {
        let id_uuid = user_id.as_uuid();
        let handle_str = profile.handle.as_str();
        let name_str = profile.display_name.as_str();
        let bio_str = profile.bio.as_ref().map(Bio::as_str);

        let row = sqlx::query!(
            r#"
            UPDATE "user"
            SET handle = $2,
                display_name = $3,
                bio = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING id, account_id, handle, display_name, bio, avatar_public_id, created_at, updated_at
            "#,
            id_uuid,
            handle_str,
            name_str,
            bio_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?
        .ok_or_else(|| RepoError::NotFound {
            entity: "user",
            id: user_id.to_string(),
        })?;

        Self::map_row(UserDbRow {
            id: row.id,
            account_id: row.account_id,
            handle: row.handle,
            display_name: row.display_name,
            bio: row.bio,
            avatar_public_id: row.avatar_public_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError> {
        let hash_str = token_hash.as_str();
        let row = sqlx::query!(
            r#"
            SELECT u.id, u.account_id, u.handle, u.display_name, u.bio, u.avatar_public_id, u.created_at, u.updated_at
            FROM session s
            JOIN "user" u ON s.account_id = u.account_id
            WHERE s.token_hash = $1 AND s.expires_at > now()
            "#,
            hash_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(UserDbRow {
                id: r.id,
                account_id: r.account_id,
                handle: r.handle,
                display_name: r.display_name,
                bio: r.bio,
                avatar_public_id: r.avatar_public_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
        })
        .transpose()
    }

    async fn update_avatar(
        &self,
        user_id: UserId,
        avatar_public_id: Option<&haven_domain::image::ImagePublicId>,
    ) -> Result<User, RepoError> {
        let id_uuid = user_id.as_uuid();
        let avatar_str = avatar_public_id.map(haven_domain::image::ImagePublicId::as_str);

        let row = sqlx::query!(
            r#"
            UPDATE "user"
            SET avatar_public_id = $2,
                updated_at = now()
            WHERE id = $1
            RETURNING id, account_id, handle, display_name, bio, avatar_public_id, created_at, updated_at
            "#,
            id_uuid,
            avatar_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?
        .ok_or_else(|| RepoError::NotFound {
            entity: "user",
            id: user_id.to_string(),
        })?;

        Self::map_row(UserDbRow {
            id: row.id,
            account_id: row.account_id,
            handle: row.handle,
            display_name: row.display_name,
            bio: row.bio,
            avatar_public_id: row.avatar_public_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
