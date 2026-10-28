//! Assemble validated facets, retained form values, summaries and navigation links.

use super::super::FeedQuery;
use crate::app::components::{chip_row::ChipRowData, feed_filters::FeedFiltersData};
use haven_domain::offer::{Category, PublicCriteria, PublicCriterion};

#[derive(Clone, Copy)]
struct FeedLink<'a> {
    scope: &'a str,
    category: &'a str,
    after: Option<&'a str>,
}

pub(in crate::app::offers::feed) struct ParsedCriteria {
    pub(in crate::app::offers::feed) criteria: PublicCriteria,
    pub(in crate::app::offers::feed) controls: FeedFiltersData,
}

impl ParsedCriteria {
    pub(in crate::app::offers::feed) fn parse(query: Option<&FeedQuery>) -> Self {
        let raw_scope = query
            .and_then(|query| query.scope.as_deref())
            .filter(|scope| !scope.is_empty())
            .unwrap_or("public");
        let scope_error = super::super::collection::Collection::parse(raw_scope)
            .is_none()
            .then(|| "Choose a valid collection.".to_string());
        let raw_category = query
            .and_then(|q| q.category.as_deref())
            .unwrap_or_default();
        let (category, category_error) = super::category::parse(raw_category);
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
        let criteria = PublicCriteria {
            category,
            title,
            price,
        };
        let mut parts = Vec::new();
        for criterion in criteria.iter() {
            match criterion {
                // The selected chip already communicates the topic.
                PublicCriterion::Category(_) => {}
                PublicCriterion::Title(title) => super::title::summarize(title, &mut parts),
                PublicCriterion::Price(price) => super::price::summarize(price, &mut parts),
            }
        }
        let mut clear_serializer = url::form_urlencoded::Serializer::new(String::new());
        append_scope(&mut clear_serializer, raw_scope);
        super::category::append_url(&mut clear_serializer, raw_category);
        let category_query = clear_serializer.finish();
        let mut clear_serializer = url::form_urlencoded::Serializer::new(String::new());
        append_scope(&mut clear_serializer, raw_scope);
        super::category::append_url(&mut clear_serializer, raw_category);
        super::title::append_url(&mut clear_serializer, raw_title);
        let clear_query = clear_serializer.finish();
        let controls = FeedFiltersData {
            scope: raw_scope.to_string(),
            scope_error,
            scope_row: None,
            category: raw_category.to_owned(),
            category_error,
            clear_filters_href: if category_query.is_empty() {
                "/offers".to_string()
            } else {
                format!("/offers?{category_query}")
            },
            title_query: raw_title.to_owned(),
            currency: raw_currency.to_owned(),
            min_price: min,
            max_price: max,
            title_error,
            price_error,
            price_filter_count: u32::from(criteria.price.is_some()),
            summary: (!parts.is_empty()).then(|| parts.join(" · ")),
            clear_price_href: if clear_query.is_empty() {
                "/offers".to_string()
            } else {
                format!("/offers?{clear_query}")
            },
        };
        let mut parsed = Self { criteria, controls };
        parsed.controls.scope_row = Some(super::super::collection::row(&parsed));
        parsed
    }

    pub(in crate::app::offers::feed) fn href(&self, after: Option<&str>) -> String {
        self.href_to(FeedLink {
            scope: &self.controls.scope,
            category: &self.controls.category,
            after,
        })
    }

    pub(in crate::app::offers::feed) fn category_row(&self) -> ChipRowData {
        super::category::row(self)
    }

    pub(in crate::app::offers::feed) fn category_href(&self, category: Option<Category>) -> String {
        self.href_to(FeedLink {
            scope: &self.controls.scope,
            category: category.map_or("", Category::as_str),
            after: None,
        })
    }

    pub(in crate::app::offers::feed) fn scope_href(
        &self,
        scope: super::super::collection::Collection,
    ) -> String {
        self.href_to(FeedLink {
            scope: scope.label(),
            category: &self.controls.category,
            after: None,
        })
    }

    fn href_to(&self, target: FeedLink<'_>) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        append_scope(&mut serializer, target.scope);
        super::category::append_url(&mut serializer, target.category);
        super::title::append_url(&mut serializer, &self.controls.title_query);
        super::price::append_url(&mut serializer, &self.controls);
        if let Some(after) = target.after {
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

fn append_scope(serializer: &mut url::form_urlencoded::Serializer<'_, String>, scope: &str) {
    if scope != "public" {
        serializer.append_pair("scope", scope);
    }
}
