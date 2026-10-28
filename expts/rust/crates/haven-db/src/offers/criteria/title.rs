//! PostgreSQL full-text title matching, with query text bound as data.

use haven_domain::offer::SearchQuery;
use sqlx::{Postgres, QueryBuilder};

pub(super) fn append(sql: &mut QueryBuilder<Postgres>, title: &SearchQuery) {
    sql.push(" AND o.title_search @@ plainto_tsquery('english', ")
        .push_bind(title.as_str().to_owned())
        .push(")");
}
