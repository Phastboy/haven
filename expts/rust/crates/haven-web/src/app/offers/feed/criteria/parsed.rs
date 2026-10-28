//! Assemble the existing criteria without changing accepted input or rendered controls.

use super::super::FeedQuery;
use crate::app::components::feed_filters::{FeedFiltersData, clear_price_href};
use haven_domain::offer::{PublicCriteria, PublicCriterion};

pub(in crate::app::offers::feed) struct ParsedCriteria {
    pub(in crate::app::offers::feed) criteria: PublicCriteria,
    pub(in crate::app::offers::feed) controls: FeedFiltersData,
}

impl ParsedCriteria {
    pub(in crate::app::offers::feed) fn parse(query: Option<&FeedQuery>) -> Self {
        let raw_title = query
            .and_then(|q| q.q.as_deref())
            .unwrap_or_default()
            .trim();
        let raw_currency = query
            .and_then(|q| q.currency.as_deref())
            .unwrap_or_default()
            .trim();
        let min = query.and_then(|q| q.min_price);
        let max = query.and_then(|q| q.max_price);
        let (title, title_error) = super::title::parse(raw_title);
        let (price, price_error) = super::price::parse(raw_currency, min, max);
        let criteria = PublicCriteria { title, price };
        let mut parts = Vec::new();
        for criterion in criteria.iter() {
            match criterion {
                PublicCriterion::Title(title) => super::title::summarize(title, &mut parts),
                PublicCriterion::Price(price) => super::price::summarize(price, &mut parts),
            }
        }
        let controls = FeedFiltersData {
            title_query: raw_title.to_owned(),
            currency: raw_currency.to_owned(),
            min_price: min,
            max_price: max,
            title_error,
            price_error,
            price_filter_count: u32::from(criteria.price.is_some()),
            summary: (!parts.is_empty()).then(|| parts.join(" · ")),
            clear_price_href: clear_price_href(raw_title),
        };
        Self { criteria, controls }
    }

    pub(in crate::app::offers::feed) fn href(&self, after: Option<&str>) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        super::title::append_url(&mut serializer, &self.controls.title_query);
        super::price::append_url(&mut serializer, &self.controls);
        if let Some(after) = after {
            serializer.append_pair("after", after);
        }
        let query = serializer.finish();
        if query.is_empty() {
            "/offers".to_string()
        } else {
            format!("/offers?{query}")
        }
    }
}
