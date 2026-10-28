//! Compose the validated facets without accepting SQL from callers.

use haven_domain::offer::{PublicCriteria, PublicCriterion};
use sqlx::{Postgres, QueryBuilder};

pub(in crate::offers) fn append(sql: &mut QueryBuilder<Postgres>, criteria: &PublicCriteria) {
    for criterion in criteria.iter() {
        match criterion {
            PublicCriterion::Title(title) => super::title::append(sql, title),
            PublicCriterion::Price(price) => super::price::append(sql, price),
        }
    }
}
