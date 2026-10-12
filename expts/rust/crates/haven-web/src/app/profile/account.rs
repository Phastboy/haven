//! Account and session management route at `/profile/account`.

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

#[topcoat::router::page]
pub async fn account_page(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    let handle_str = format!("@{}", user.handle.as_str());
    let display_name_str = user.display_name.as_str().to_string();

    Ok(view! {
        <div class="profile-account-page">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <h2 class="section-title">"Account & Session"</h2>
                <p class="text-muted">"Manage your active session on Haven."</p>
            </div>

            <div class="account-card">
                <div class="account-detail-group">
                    <span class="detail-label text-muted">"Signed in as"</span>
                    <strong class="detail-value">(display_name_str) " " <span class="text-muted">"(" (handle_str) ")"</span></strong>
                </div>

                <div class="sign-out-section">
                    <form action="/auth/sign-out" method="post" class="sign-out-form">
                        <button type="submit" class="btn btn-secondary">
                            "Sign Out"
                        </button>
                    </form>
                </div>
            </div>
        </div>
    })
}
