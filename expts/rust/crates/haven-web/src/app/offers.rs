pub mod fields;
pub mod id;
pub mod new;

use topcoat::{Result as TopcoatResult, context::Cx, view::view};

#[topcoat::router::page]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::cx_helpers::require_auth(cx).await?;
    let offers = crate::cx_helpers::registry(cx)
        .offers()
        .find_by_user(user.id)
        .await
        .map_err(crate::cx_helpers::map_repo_err)?;

    Ok(view! {
        <div class="offers">
            <h1>"Your Offers"</h1>
            <a href="/offers/new" class="button">"New Offer"</a>
            <ul>
                for offer in offers {
                    <li>
                        <a href=(format!("/offers/{}", offer.id))>
                            (offer.title)
                        </a>
                    </li>
                }
            </ul>
        </div>
    })
}
