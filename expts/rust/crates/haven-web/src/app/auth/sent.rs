use crate::app::components::button::{ButtonVariant, button_link};
use topcoat::{Result as TopcoatResult, context::Cx, view::view};

#[topcoat::router::page]
pub async fn sent_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let email_delivery_enabled = crate::app::state::email_delivery_enabled(cx);
    Ok(view! {
        <div class="auth-card">
            if email_delivery_enabled {
                <h1>"Check your email"</h1>
                <p class="text-muted">"If an account exists for that email, we've sent a magic link."</p>
            } else {
                <h1>"Email delivery unavailable"</h1>
                <p class="text-muted">
                    "Email delivery infrastructure is currently unconfigured. Please return to sign in with Google."
                </p>
            }
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
