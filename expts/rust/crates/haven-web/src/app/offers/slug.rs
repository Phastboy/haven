use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

topcoat::router::module_param!(slug);

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
        "\"{:x}-{:x}\"",
        offer.id.as_uuid().as_u128(),
        offer.updated_at.timestamp_micros()
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

    let not_modified =
        if let Some(inm) = topcoat::router::request::headers(cx).get(http::header::IF_NONE_MATCH) {
            inm.as_bytes() == etag.as_bytes() || inm.as_bytes() == format!("W/{etag}").as_bytes()
        } else {
            false
        };

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
