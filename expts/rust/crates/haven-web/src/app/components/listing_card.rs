//! Compact square-image card used by the public offers feed.

use haven_domain::offer::OfferSlug;
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

use super::save_heart::{SaveHeartData, save_heart};

/// All display data required by a public offer card.
pub struct ListingCardData {
    /// Offer title.
    pub title: String,
    /// Offer detail URL.
    pub href: String,
    /// Display-ready price.
    pub price_text: String,
    /// Maker's display name.
    pub maker_name: String,
    /// Maker profile URL.
    pub maker_href: String,
    /// Square cover image URL.
    pub cover_url: Option<String>,
    /// Square cover image srcset.
    pub cover_srcset: Option<String>,
    /// Whether this offer is saved by the viewer.
    pub is_saved: bool,
    /// Feed URL to return to after saving.
    pub return_to: String,
    /// Whether the viewer is signed in.
    pub is_authenticated: bool,
    /// Whether the image is near the top of the page.
    pub is_above_fold: bool,
    /// Validated offer slug.
    pub slug: OfferSlug,
}

/// Renders one public offer card.
#[component]
pub async fn listing_card(props: ListingCardData) -> TopcoatResult<impl View> {
    let loading = if props.is_above_fold { "eager" } else { "lazy" };
    Ok(
        view! { <li id=(format!("offer-{}", props.slug.as_str())) class="listing-card">
            <div class="listing-card-media">
                <span class="cover-fallback" aria-hidden="true" role="img" aria-label="Image unavailable">"Image unavailable"</span>
                if let Some(src) = props.cover_url {
                    <img class="listing-card-image" data-cover-image="" width="320" height="320" src=(src) srcset=(props.cover_srcset.unwrap_or_default()) alt="" loading=(loading) />
                } else {
                    <div class="listing-card-placeholder" aria-hidden="true"></div>
                }
                <div class="listing-card-save">
                    save_heart(props: SaveHeartData { slug: props.slug.clone(), is_saved: props.is_saved,
                        return_to: Some(props.return_to), show_label: false, is_authenticated: props.is_authenticated })
                </div>
            </div>
            <div class="listing-card-copy">
                <h2 class="listing-card-title"><a class="listing-card-link" href=(props.href)>(props.title)</a></h2>
                <p class="listing-card-maker"><a href=(props.maker_href)>(props.maker_name)</a></p>
                <p class="listing-card-price">(props.price_text)</p>
            </div>
        </li> },
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "component test fixtures and assertions")]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    async fn listing_card_renders_escaped_content_and_square_image() {
        let cx = Cx::default();
        let __cx = &cx;
        let card = view! { listing_card(props: ListingCardData {
            title: "<Wood chair>".into(), href: "/offers/wood-chair".into(), price_text: "Free".into(),
            maker_name: "Maker & Co".into(), maker_href: "/profiles/maker".into(),
            cover_url: Some("https://res.cloudinary.com/haven/image/private/t_card_sq/offers/chair".into()),
            cover_srcset: None, is_saved: false, return_to: "/offers".into(),
            is_authenticated: false, is_above_fold: true, slug: OfferSlug::parse("wood-chair").expect("valid test slug"),
        }) };
        let html = card.single().await.expect("single component").render(__cx);
        assert!(html.contains("listing-card-image"));
        assert!(html.contains("data-cover-image"));
        assert!(html.contains("Image unavailable"));
        assert!(html.contains("&lt;Wood chair&gt;"));
        assert!(html.contains("Maker &amp; Co"));
        assert!(html.contains("loading=\"eager\""));
    }
}
