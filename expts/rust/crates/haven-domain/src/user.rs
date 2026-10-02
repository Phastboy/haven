use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::account::AccountId;
use crate::offer::UserId;

/// Platform participant (authz + platform context).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub account_id: AccountId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
