//! Bound topic predicate, applied before pagination.

use haven_domain::offer::Category;
use sqlx::{Postgres, QueryBuilder};

pub(super) fn append(sql: &mut QueryBuilder<Postgres>, category: Category) {
    sql.push(" AND o.category = ").push_bind(category.as_str());
}
