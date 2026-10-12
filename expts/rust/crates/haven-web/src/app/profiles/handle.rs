//! Public profile page by handle: `/profiles/{handle}`.

use crate::app::components::empty_state::empty_state;
use crate::app::components::identity_block::{IdentityBlockData, identity_block};
use crate::app::components::offer_card::offer_card;
use crate::app::components::pagination::pagination_nav;
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

topcoat::router::module_param!(handle);

/// Keyset pagination query parameter for a person's public offers list.
#[topcoat::router::query_params]
pub struct ProfileQuery {
    /// Opaque cursor string marking the offer to list after.
    pub after: Option<String>,
}

#[topcoat::router::page]
pub async fn view_profile(cx: &Cx) -> TopcoatResult<impl View> {
    let handle_raw: &str = topcoat::router::path_param::<Handle>(cx);
    let profile_handle = haven_domain::user::ProfileHandle::parse(handle_raw)
        .map_err(|_| topcoat::router::error::not_found())?;

    let registry = crate::app::state::registry(cx);
    let user = registry
        .users()
        .find_by_handle(&profile_handle)
        .await
        .map_err(crate::app::state::map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;

    let query = topcoat::router::query_params::<ProfileQuery>(cx).ok();
    let after_cursor = query
        .and_then(|q| q.after.as_deref())
        .and_then(|raw| haven_domain::offer::OfferCursor::decode(raw).ok());

    let page = registry
        .offers()
        .list_by_user(user.id, after_cursor.as_ref(), 20)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    let next_href = page
        .next_cursor
        .map(|c| format!("/profiles/{}?after={c}", user.handle.as_str()));

    let bio_text = user.bio.as_ref().map(|b| b.as_str().to_string());
    let offers_heading = format!("Offers by {}", user.display_name.as_str());
    let joined_date = user.created_at.format("Joined %B %Y").to_string();

    let identity_data = IdentityBlockData {
        display_name: user.display_name.as_str().to_string(),
        handle: user.handle.as_str().to_string(),
        joined_date,
        avatar_url: None,
        bio: bio_text,
    };

    Ok(view! {
        <div class="profile-page">
            identity_block(data: identity_data)

            <section class="profile-offers">
                <h2 class="section-title">(offers_heading)</h2>
                if page.items.is_empty() {
                    empty_state(
                        title: "No offers yet",
                        message: "This person has not posted any offers yet. Check back later.",
                    )
                } else {
                    <ul class="offer-list" role="list">
                        for offer in page.items {
                            offer_card(
                                title: offer.title.clone(),
                                href: format!("/offers/{}", offer.slug),
                                price_text: if offer.price.as_i32() == 0 {
                                    "Free".to_string()
                                } else {
                                    format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
                                },
                                description: offer.description.clone(),
                            )
                        }
                    </ul>
                    if next_href.is_some() {
                        pagination_nav(
                            prev_href: None,
                            next_href: next_href,
                        )
                    }
                }
            </section>
        </div>
    })
}
