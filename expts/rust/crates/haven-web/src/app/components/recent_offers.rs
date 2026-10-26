use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

use super::listing_card::{ListingCardData, listing_card};

pub struct RecentOffersData {
    pub offers: Vec<ListingCardData>,
}

/// Renders the public landing page's latest offers, or nothing when empty.
#[component]
pub async fn recent_offers(props: RecentOffersData) -> TopcoatResult<impl View> {
    Ok(view! {
        if !props.offers.is_empty() {
            <section class="landing-recent" aria-labelledby="landing-recent-title">
                <div class="landing-recent-heading">
                    <h2 id="landing-recent-title">"Recently offered"</h2>
                    <a href="/offers" class="landing-browse-all">
                        <span class="landing-browse-all-desktop">"Browse all offers"</span>
                        <span class="landing-browse-all-mobile">"Browse all"</span>
                        <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                    </a>
                </div>
                <ul class="landing-offers-grid" role="list">
                    for offer in props.offers {
                        listing_card(props: offer)
                    }
                </ul>
            </section>
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use haven_domain::offer::OfferSlug;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[allow(clippy::expect_used, reason = "component render test fixture")]
    fn card() -> ListingCardData {
        ListingCardData {
            title: "Handmade chair".into(),
            href: "/offers/handmade-chair".into(),
            price_text: "20 NGN".into(),
            maker_name: "A Maker".into(),
            maker_href: "/profiles/a-maker".into(),
            cover_url: None,
            cover_srcset: None,
            is_saved: false,
            return_to: "/".into(),
            is_authenticated: false,
            is_above_fold: true,
            slug: OfferSlug::parse("handmade-chair").expect("valid test slug"),
        }
    }

    #[tokio::test]
    #[allow(clippy::expect_used, reason = "component render test")]
    async fn recent_offers_renders_heading_card_count_and_maker() {
        let cx = Cx::default();
        let __cx = &cx;
        let element =
            view! { recent_offers(props: RecentOffersData { offers: vec![card(), card()] }) };
        let html = element.single().await.expect("single section").render(__cx);
        assert!(html.contains("Recently offered"));
        assert!(html.contains("offer-handmade-chair"));
        assert_eq!(html.matches("class=\"listing-card\"").count(), 2);
        assert!(html.contains("A Maker"));
        assert!(html.contains("href=\"/auth/sign-in\""));
    }

    #[tokio::test]
    #[allow(clippy::expect_used, reason = "component render test")]
    async fn recent_offers_renders_nothing_when_empty() {
        let cx = Cx::default();
        let __cx = &cx;
        let element = view! { recent_offers(props: RecentOffersData { offers: Vec::new() }) };
        let html = element
            .single()
            .await
            .expect("single empty view")
            .render(__cx);
        assert!(!html.contains("Recently offered"));
        assert!(!html.contains("landing-recent"));
    }
}
