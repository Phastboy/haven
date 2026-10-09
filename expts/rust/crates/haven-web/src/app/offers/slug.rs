use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

topcoat::router::module_param!(slug);

const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

fn weak_etag_match(header_tag: &str, current_etag: &str) -> bool {
    let current_opaque = current_etag.strip_prefix("W/").unwrap_or(current_etag);
    let header_opaque = header_tag.strip_prefix("W/").unwrap_or(header_tag);
    current_opaque == header_opaque
}

fn matches_if_none_match(inm_header: &str, current_etag: &str) -> bool {
    for raw_tag in inm_header.split(',') {
        let tag = raw_tag.trim();
        if tag.is_empty() {
            continue;
        }
        if tag == "*" || weak_etag_match(tag, current_etag) {
            return true;
        }
    }
    false
}

#[topcoat::router::page]
pub async fn view_slug(cx: &Cx) -> TopcoatResult<impl View> {
    let slug_raw: &str = topcoat::router::path_param::<Slug>(cx);
    let offer_slug = haven_domain::offer::OfferSlug::parse(slug_raw)
        .map_err(|_| topcoat::router::error::not_found())?;

    let repo = crate::cx_helpers::registry(cx).offers();
    let offer = repo
        .find_by_slug(&offer_slug)
        .await
        .map_err(crate::cx_helpers::map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;

    // Shared caching policy with ETag
    let etag = format!(
        "\"{:x}-{:x}-v{}\"",
        offer.id.as_uuid().as_u128(),
        offer.updated_at.timestamp_micros(),
        BUILD_VERSION
    );

    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        topcoat::router::header::HeaderName::from_static("cache-control"),
        topcoat::router::header::HeaderValue::from_static("public, max-age=60, s-maxage=300"),
    );
    if let Ok(etag_val) = topcoat::router::header::HeaderValue::from_str(&etag) {
        headers.append(
            topcoat::router::header::HeaderName::from_static("etag"),
            etag_val,
        );
    }

    let not_modified = topcoat::router::request::headers(cx)
        .get_all(http::header::IF_NONE_MATCH)
        .iter()
        .any(|val| {
            val.to_str()
                .is_ok_and(|header_str| matches_if_none_match(header_str, &etag))
        });

    let price_text = if offer.price.as_i32() == 0 {
        "Free".to_string()
    } else {
        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
    };

    let canonical_url = format!("/offers/{}", offer.slug);

    Ok(view! {
        if not_modified {
            (http::StatusCode::NOT_MODIFIED)
        } else {
            <div class="offer-public-page">
                <link rel="canonical" href=(canonical_url) />
                <div class="page-header">
                    <a href="/offers" class="text-muted back-link">"← Back to all offers"</a>
                    <h1 class="offer-title">(offer.title)</h1>
                    <div class="offer-price-badge">(price_text)</div>
                </div>
                <div class="offer-body">
                    <p class="offer-description-full">(offer.description)</p>
                </div>
            </div>
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_etag_match() {
        assert!(weak_etag_match("\"tag\"", "\"tag\""));
        assert!(weak_etag_match("W/\"tag\"", "\"tag\""));
        assert!(weak_etag_match("\"tag\"", "W/\"tag\""));
        assert!(weak_etag_match("W/\"tag\"", "W/\"tag\""));
        assert!(!weak_etag_match("\"other\"", "\"tag\""));
    }

    #[test]
    fn test_matches_if_none_match() {
        let current = "\"123-456-v0.1.0\"";
        assert!(matches_if_none_match("*", current));
        assert!(matches_if_none_match("\"123-456-v0.1.0\"", current));
        assert!(matches_if_none_match("W/\"123-456-v0.1.0\"", current));
        assert!(matches_if_none_match(
            "\"other\", W/\"123-456-v0.1.0\"",
            current
        ));
        assert!(matches_if_none_match(
            " \"other\" ,  \"123-456-v0.1.0\" ",
            current
        ));
        assert!(!matches_if_none_match("\"other\", \"second\"", current));
        assert!(!matches_if_none_match("", current));
        assert!(!matches_if_none_match(" , ", current));
    }
}
