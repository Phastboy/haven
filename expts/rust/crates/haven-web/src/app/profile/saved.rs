//! Owner's saved offers at `/profile/saved`.
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

#[topcoat::router::query_params]
pub struct SavedQuery {
    pub after: Option<String>,
}

#[topcoat::router::page]
pub async fn saved(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let query = topcoat::router::query_params::<SavedQuery>(cx).ok();
    let cursor = query
        .as_ref()
        .and_then(|q| q.after.as_deref())
        .and_then(haven_domain::saved::CollectionCursor::decode);
    let page = crate::app::state::registry(cx)
        .saved_offers()
        .list(user.id, cursor, 20)
        .await
        .map_err(crate::app::state::map_repo_err)?;
    Ok(
        view! { <section><meta name="robots" content="noindex" /><h1>"Saved offers"</h1>
            if page.items.is_empty() { <p>"No saved offers yet."</p> } else {
                <ul class="offer-list" role="list">
                    for item in page.items.iter() {
                        if let Some(offer) = item.offer.as_ref() {
                            <li><a href=(format!("/offers/{}", offer.slug.as_str()))>(offer.title.clone())</a></li>
                        } else { <li><span>"Offer no longer available"</span></li> }
                    }
                </ul>
            }
            if let Some(cursor) = page.next_cursor { <a href=(format!("/profile/saved?after={cursor}")) rel="next">"Next"</a> }
        </section> },
    )
}
