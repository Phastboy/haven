//! Public offers feed page and query parameters.

use crate::app::components::empty_state::empty_state;
use crate::app::components::feed_filters::{FeedFiltersData, clear_price_href, feed_filters};
use crate::app::components::listing_card::{ListingCardData, listing_card};
use crate::app::components::pagination::pagination_nav;
use haven_domain::offer::{
    CurrencyCode, OfferCursor, OfferFilter, Price, PriceFilter, PublicFeedQuery, SearchQuery,
};
use topcoat::{Result as TopcoatResult, context::Cx, view::view};

/// Keyset pagination and filter query parameters for the public offers feed.
#[topcoat::router::query_params]
pub struct FeedQuery {
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
    let after_cursor = query
        .as_ref()
        .and_then(|q| q.after.as_deref())
        .and_then(|raw| OfferCursor::decode(raw).ok());

    let raw_q = query
        .as_ref()
        .and_then(|q| q.q.as_deref())
        .unwrap_or_default()
        .trim();
    let raw_currency = query
        .as_ref()
        .and_then(|q| q.currency.as_deref())
        .unwrap_or_default()
        .trim();
    let raw_min_price = query.as_ref().and_then(|q| q.min_price);
    let raw_max_price = query.as_ref().and_then(|q| q.max_price);

    let mut title_error = None;
    let title_query = if raw_q.is_empty() {
        None
    } else {
        match SearchQuery::parse(raw_q) {
            Ok(sq) => Some(sq),
            Err(e) => {
                title_error = Some(e.to_string());
                None
            }
        }
    };

    let mut price_error = None;
    let price_filter = if raw_min_price.is_some() || raw_max_price.is_some() {
        if raw_currency.is_empty() {
            price_error = Some("Currency is required when filtering by price.".to_string());
            None
        } else if let Ok(curr) = CurrencyCode::parse(raw_currency) {
            let min_p = raw_min_price.map(Price::new).transpose();
            let max_p = raw_max_price.map(Price::new).transpose();

            match (min_p, max_p) {
                (Err(_), _) | (_, Err(_)) => {
                    price_error = Some("Price must be non-negative.".to_string());
                    None
                }
                (Ok(min_opt), Ok(max_opt)) => match PriceFilter::new(curr, min_opt, max_opt) {
                    Ok(pf) => Some(pf),
                    Err(e) => {
                        price_error = Some(e.to_string());
                        None
                    }
                },
            }
        } else {
            price_error = Some("Invalid currency code.".to_string());
            None
        }
    } else {
        None
    };

    let filter = OfferFilter {
        title: title_query,
        price: price_filter,
    };

    let active_filters_count =
        usize::from(filter.title.is_some()).saturating_add(usize::from(filter.price.is_some()));

    let mut summary_parts = Vec::new();
    if let Some(title) = filter.title.as_ref() {
        summary_parts.push(format!("Search: {}", title.as_str()));
    }
    if let Some(price) = filter.price {
        summary_parts.push(price.currency.as_str().to_string());
        if let Some(min) = price.min {
            summary_parts.push(format!("Min {}", min.as_i32()));
        }
        if let Some(max) = price.max {
            summary_parts.push(format!("Max {}", max.as_i32()));
        }
    }
    let filter_summary = (!summary_parts.is_empty()).then(|| summary_parts.join(" · "));
    let price_filter_count = u32::from(filter.price.is_some());

    let feed_query = PublicFeedQuery::new(filter, after_cursor, 20);

    let repo = crate::app::state::registry(cx).offers();
    let page = repo
        .list_public(&feed_query)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    let next_href = page.next_cursor.map(|c| {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if !raw_q.is_empty() {
            serializer.append_pair("q", raw_q);
        }
        if !raw_currency.is_empty() {
            serializer.append_pair("currency", raw_currency);
        }
        if let Some(min) = raw_min_price {
            serializer.append_pair("min_price", &min.to_string());
        }
        if let Some(max) = raw_max_price {
            serializer.append_pair("max_price", &max.to_string());
        }
        serializer.append_pair("after", &c);
        format!("/offers?{}", serializer.finish())
    });

    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let viewer = crate::app::auth::guard::current_user(cx).await?;
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
    let mut return_serializer = url::form_urlencoded::Serializer::new(String::new());
    if !raw_q.is_empty() {
        return_serializer.append_pair("q", raw_q);
    }
    if !raw_currency.is_empty() {
        return_serializer.append_pair("currency", raw_currency);
    }
    if let Some(min) = raw_min_price {
        return_serializer.append_pair("min_price", &min.to_string());
    }
    if let Some(max) = raw_max_price {
        return_serializer.append_pair("max_price", &max.to_string());
    }
    let return_query = return_serializer.finish();
    let return_to = format!(
        "/offers{}",
        if return_query.is_empty() {
            String::new()
        } else {
            format!("?{return_query}")
        }
    );

    Ok(view! {
        <div class="offers-feed">
            feed_filters(data: FeedFiltersData {
                title_query: raw_q.to_string(),
                currency: raw_currency.to_string(),
                min_price: raw_min_price,
                max_price: raw_max_price,
                title_error,
                price_error,
                price_filter_count,
                summary: filter_summary,
                clear_price_href: clear_price_href(raw_q),
            })
            if page.items.is_empty() {
                if active_filters_count > 0 {
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
