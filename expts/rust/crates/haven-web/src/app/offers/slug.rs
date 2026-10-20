#![allow(
    clippy::too_many_lines,
    reason = "Offer detail view with photo stack, maker attribution, and ETag caching"
)]

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

pub mod message;

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

    let registry = crate::app::state::registry(cx);
    let offer = registry
        .offers()
        .find_by_slug(&offer_slug)
        .await
        .map_err(crate::app::state::map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;

    let is_owner = match crate::app::auth::guard::current_user(cx).await {
        Ok(Some(u)) => u.id == offer.user_id,
        _ => false,
    };

    if !offer.is_public() && !is_owner {
        return Err(topcoat::router::error::not_found().into());
    }

    let maker = registry
        .users()
        .find_by_id(offer.user_id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    // Shared caching policy with ETag (only for public views)
    let etag = format!(
        "\"{:x}-{:x}-v{}\"",
        offer.id.as_uuid().as_u128(),
        offer.updated_at.timestamp_micros(),
        BUILD_VERSION
    );

    let headers = topcoat::router::response::response_headers(cx);
    if is_owner {
        headers.append(
            topcoat::router::header::HeaderName::from_static("cache-control"),
            topcoat::router::header::HeaderValue::from_static("private, no-cache"),
        );
    } else {
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
    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let total_images = offer.images.len();
    let offer_title = offer.title.clone();

    Ok(view! {
        if not_modified {
            (http::StatusCode::NOT_MODIFIED)
        } else {
            <div class="offer-detail">
                <link rel="canonical" href=(canonical_url) />
                <div class="page-header">
                    <div class="offer-detail-header">
                        <h1 class="offer-detail-title">(offer_title.clone())</h1>
                        <div class="offer-detail-price">(price_text)</div>
                    </div>
                    if let Some(user) = maker {
                        <div class="offer-detail-maker">
                            <span class="text-muted">"Offered by "</span>
                            <a href=(format!("/profiles/{}", user.handle.as_str())) class="maker-link">(user.display_name.as_str())</a>
                        </div>
                    }
                </div>
                if total_images > 0 {
                    <div class="offer-photo-stack">
                        for (idx, img) in offer.images.iter().enumerate() {
                            let alt_text = format!("{}, photo {} of {}", offer_title, idx + 1, total_images);
                            <div class="offer-photo-item">
                                <img
                                    src=(crate::app::components::image::detail_url(cloud_name, img.public_id.as_str()))
                                    srcset=(crate::app::components::image::detail_srcset(cloud_name, img.public_id.as_str()))
                                    alt=(alt_text)
                                    class="offer-detail-image"
                                    width="640"
                                    height="480"
                                    loading=(if idx == 0 { "eager" } else { "lazy" })
                                />
                            </div>
                        }
                    </div>
                }
                <div class="offer-detail-body">
                    <p>(offer.description)</p>
                </div>

                if !is_owner {
                    <div class="offer-inquiry-action">
                        <h3>"Message about this offer"</h3>
                        <form action=(format!("/offers/{}/message", offer.slug.as_str())) method="post" class="offer-message-form">
                            <textarea name="body" placeholder="Ask a question or express your interest..." rows="3" required="required" maxlength="2000"></textarea>
                            <div>
                                <button type="submit" class="btn btn-primary">"Send Message"</button>
                            </div>
                        </form>
                    </div>
                }
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
