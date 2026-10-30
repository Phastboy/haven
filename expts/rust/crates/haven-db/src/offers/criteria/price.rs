//! Currency and inclusive price bounds, with all values bound as data.

use haven_domain::offer::PriceFilter;
use sqlx::{Postgres, QueryBuilder};

pub(super) fn append(sql: &mut QueryBuilder<Postgres>, price: &PriceFilter) {
    sql.push(" AND o.currency = ")
        .push_bind(price.currency.as_str());
    if let Some(min) = price.min {
        sql.push(" AND o.price >= ").push_bind(min.as_i32());
    }
    if let Some(max) = price.max {
        // Prices are nonnegative (check_price_non_negative), so a zero upper
        // bound is equality. This lets the currency/price/feed index stream
        // free offers in keyset order instead of scanning and sorting them.
        let predicate = if max.as_i32() == 0 {
            " AND o.price = "
        } else {
            " AND o.price <= "
        };
        sql.push(predicate).push_bind(max.as_i32());
    }
}
