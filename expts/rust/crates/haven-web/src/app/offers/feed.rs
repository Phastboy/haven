//! Public offers feed page and query parameters.

mod criteria;

use crate::app::components::chip_row::chip_row;
use crate::app::components::empty_state::empty_state;

use crate::app::components::feed_filters::feed_filters;
use crate::app::components::listing_card::{ListingCardData, listing_card};
use crate::app::components::pagination::pagination_nav;
use criteria::ParsedCriteria;
use haven_domain::offer::{OfferCursor, PublicFeedQuery};
use topcoat::{Result as TopcoatResult, context::Cx, view::view};

/// Keyset pagination and filter query parameters for the public offers feed.
#[topcoat::router::query_params]
pub struct FeedQuery {
    /// Stable topic slug; unknown values are validation errors.
    pub category: Option<String>,
    /// Opaque cursor string marking the offer to list after.
    pub after: Option<String>,
    /// Search query string matching offer title.
    pub q: Option<String>,
    /// ISO 4217 currency code.
    pub currency: Option<String>,
    /// Minimum price in minor units.
    pub min_price: Option<i32>,
    /// Maximum price in minor units (0 means Free).
    pub max_price: Option<i32>,
}

/// Serves the public offers feed page at `/offers`.
#[topcoat::router::page("/offers")]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let query = topcoat::router::query_params::<FeedQuery>(cx).ok();
    let (after_cursor, return_after) =
        parse_after_cursor(query.as_ref().and_then(|q| q.after.as_deref()));

    let parsed = ParsedCriteria::parse(query);
    let active_filters_count = parsed.criteria.iter().count();
    let feed_query = PublicFeedQuery::new(parsed.criteria.clone(), after_cursor, 20);

    let invalid_category = parsed.controls.category_error.is_some();
    let category_only = parsed.criteria.category.is_some()
        && active_filters_count == 1
        && parsed.controls.title_error.is_none()
        && parsed.controls.price_error.is_none();
    let canonical = canonical(cx);
    let repo = crate::app::state::registry(cx).offers();
    let page = if invalid_category {
        haven_domain::offer::PublicOfferPage {
            items: Vec::new(),
            next_cursor: None,
        }
    } else {
        repo.list_public(&feed_query)
            .await
            .map_err(crate::app::state::map_repo_err)?
    };

    let next_href = page.next_cursor.map(|cursor| parsed.href(Some(&cursor)));

    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let viewer = crate::app::auth::guard::current_user(cx).await?;
    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        topcoat::router::header::HeaderName::from_static("vary"),
        topcoat::router::header::HeaderValue::from_static("Cookie"),
    );
    if canonical.is_none() {
        headers.append(
            topcoat::router::header::HeaderName::from_static("x-robots-tag"),
            topcoat::router::header::HeaderValue::from_static("noindex, follow"),
        );
    }
    if viewer.is_some() {
        let headers = topcoat::router::response::response_headers(cx);
        headers.append(
            topcoat::router::header::HeaderName::from_static("cache-control"),
            topcoat::router::header::HeaderValue::from_static("private, no-store"),
        );
    }
    let saved_ids = if let Some(person) = viewer.as_ref() {
        let ids: Vec<_> = page.items.iter().map(|item| item.offer.id).collect();
        crate::app::state::registry(cx)
            .saved_offers()
            .saved_among(person.id, &ids)
            .await
            .map_err(crate::app::state::map_repo_err)?
    } else {
        std::collections::HashSet::new()
    };
    let return_to = parsed.href(return_after.as_deref());
    let category_row = parsed.category_row();

    Ok(view! {
        <div class="offers-feed">
            feed_filters(data: parsed.controls)
            chip_row(props: category_row)
            if page.items.is_empty() {
                if invalid_category {
                    empty_state(
                        title: "Choose a valid category",
                        message: "Select a category above to find offers.",
                        action_href: Some("/offers".to_string()),
                        action_label: Some("Clear all filters".to_string()),
                    )
                } else if category_only {
                    empty_state(
                        title: "No offers here yet",
                        message: "There are no public offers in this category yet. Try another category or clear your filters.",
                        action_href: Some("/offers".to_string()),
                        action_label: Some("Clear all filters".to_string()),
                    )
                } else if active_filters_count > 0 {
                    empty_state(
                        title: "No offers match your filters",
                        message: "Try clearing your filters or broadening your search parameters.",
                        action_href: Some("/offers".to_string()),
                        action_label: Some("Clear all filters".to_string()),
                    )
                } else {
                    empty_state(
                        title: "No offers available yet",
                        message: "There are no offers posted on Haven right now. Check back soon or create the first one.",
                        action_href: Some("/profile/offers/new".to_string()),
                        action_label: Some("Create an Offer".to_string()),
                    )
                }
            } else {
                <ul class="listing-grid" role="list">
                    for (idx, item) in page.items.iter().enumerate() {
                        let cover = item.offer.cover_image();
                        let cover_url = cover.map(|img| crate::app::components::image::card_sq_url(cloud_name, img.public_id.as_str()));
                        let cover_srcset = cover.map(|img| crate::app::components::image::card_sq_srcset(cloud_name, img.public_id.as_str()));
                        listing_card(props: ListingCardData {
                            title: item.offer.title.clone(),
                            href: format!("/offers/{}", item.offer.slug),
                            price_text: if item.offer.price.as_i32() == 0 {
                                "Free".to_string()
                            } else {
                                format!("{} {}", item.offer.price.as_i32(), item.offer.currency.as_str())
                            },
                            maker_name: item.maker_display_name.as_str().to_string(),
                            maker_href: format!("/profiles/{}", item.maker_handle.as_str()),
                            cover_url,
                            cover_srcset,
                            is_above_fold: idx < 2,
                            is_saved: saved_ids.contains(&item.offer.id),
                            return_to: return_to.clone(),
                            is_authenticated: viewer.is_some(),
                            slug: item.offer.slug.clone(),
                        })
                    }
                </ul>
                if next_href.is_some() {
                    pagination_nav(
                        prev_href: None,
                        next_href: next_href,
                    )
                }
            }
        </div>
    })
}

/// Canonical URL for an indexable first-page public landing; other criteria are noindex.
pub(crate) fn canonical(cx: &Cx) -> Option<String> {
    let query = topcoat::router::query_params::<FeedQuery>(cx).ok();
    let parsed = ParsedCriteria::parse(query);
    if query.is_some_and(|q| q.after.is_some())
        || parsed.controls.category_error.is_some()
        || parsed.controls.title_error.is_some()
        || parsed.controls.price_error.is_some()
        || parsed.criteria.title.is_some()
        || parsed.criteria.price.is_some()
    {
        None
    } else {
        Some(parsed.criteria.category.map_or_else(
            || "/offers".to_string(),
            |category| format!("/offers?category={}", category.as_str()),
        ))
    }
}

fn parse_after_cursor(raw: Option<&str>) -> (Option<OfferCursor>, Option<String>) {
    let Some(raw) = raw else {
        return (None, None);
    };
    let Ok(cursor) = OfferCursor::decode(raw) else {
        return (None, None);
    };
    (Some(cursor), Some(raw.to_owned()))
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::parse_after_cursor;
    use chrono::{TimeZone, Utc};
    use haven_domain::{offer::OfferCursor, offer::OfferId};
    use uuid::Uuid;

    #[test]
    fn keeps_only_a_decoded_cursor_for_save_return() {
        let cursor = OfferCursor::new(
            Utc.timestamp_micros(1_700_000_000_000_000)
                .single()
                .expect("valid timestamp"),
            OfferId::from_uuid(Uuid::from_u128(1)),
        );
        let encoded = cursor.encode();

        let (decoded, return_after) = parse_after_cursor(Some(&encoded));
        assert_eq!(decoded, Some(cursor));
        assert_eq!(return_after.as_deref(), Some(encoded.as_str()));
        assert_eq!(parse_after_cursor(Some("invalid")), (None, None));
        assert_eq!(parse_after_cursor(None), (None, None));
    }
}
