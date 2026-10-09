use crate::app::components::button::{ButtonVariant, button_link};
use topcoat::{Result as TopcoatResult, context::Cx, view::view};

#[topcoat::router::page]
pub async fn sent_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let _ = cx;
    Ok(view! {
        <div class="auth-card">
            <h1>"Check your email"</h1>
            <p class="text-muted">"If an account exists for that email, we've sent a magic link."</p>
            <div class="form-actions">
                button_link(
                    href: "/auth/sign-in",
                    text: "Back to sign in",
                    variant: ButtonVariant::Secondary,
                )
            </div>
        </div>
    })
}
