//! Owner's followed people at `/profile/following`.
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

#[topcoat::router::query_params]
pub struct FollowingQuery {
    pub after: Option<String>,
}

#[topcoat::router::page]
pub async fn following(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let query = topcoat::router::query_params::<FollowingQuery>(cx).ok();
    let cursor = query
        .as_ref()
        .and_then(|q| q.after.as_deref())
        .and_then(haven_domain::saved::CollectionCursor::decode);
    let page = crate::app::state::registry(cx)
        .follows()
        .list(user.id, cursor, 20)
        .await
        .map_err(crate::app::state::map_repo_err)?;
    Ok(
        view! { <section><meta name="robots" content="noindex" /><h1>"Following"</h1>
            if page.items.is_empty() { <p>"You are not following anyone yet."</p> } else {
                <ul class="offer-list" role="list">
                    for item in page.items.iter() { <li><a href=(format!("/profiles/{}", item.person.handle.as_str()))>(item.person.display_name.as_str())</a></li> }
                </ul>
            }
            if let Some(cursor) = page.next_cursor { <a href=(format!("/profile/following?after={cursor}")) rel="next">"Next"</a> }
        </section> },
    )
}
