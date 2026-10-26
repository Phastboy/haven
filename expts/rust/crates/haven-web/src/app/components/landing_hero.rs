use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

use super::button::{ButtonVariant, button_link};

/// Introduction and primary actions for the public landing page.
#[component]
pub async fn landing_hero() -> TopcoatResult<impl View> {
    Ok(view! {
        <section class="landing-hero">
            <h1 class="landing-hero-title">"Exchange offers directly"</h1>
            <p class="landing-hero-lead">"Put something you are offering on Haven, or browse what others have made available."</p>
            <div class="landing-hero-actions">
                button_link(href: "/offers", text: "Browse offers", variant: ButtonVariant::Primary)
                button_link(href: "/auth/sign-in", text: "Sign in", variant: ButtonVariant::Secondary)
            </div>
        </section>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    #[allow(clippy::expect_used, reason = "component render test")]
    async fn landing_hero_renders_copy_and_both_actions() {
        let cx = Cx::default();
        let __cx = &cx;
        let element = view! { landing_hero() };
        let html = element.single().await.expect("single hero").render(__cx);
        assert!(html.contains("Exchange offers directly"));
        assert!(html.contains(
            "Put something you are offering on Haven, or browse what others have made available."
        ));
        assert!(html.contains("href=\"/offers\""));
        assert!(html.contains("Browse offers"));
        assert!(html.contains("href=\"/auth/sign-in\""));
        assert!(html.contains("Sign in"));
    }
}
