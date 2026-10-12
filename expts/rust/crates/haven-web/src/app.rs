use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{Body, Next, response::Response},
    view::view,
};

pub mod auth;
pub mod components;
pub mod health;
pub mod navigation;
pub mod offers;
pub mod origin;
pub mod profile;
pub mod profiles;
pub mod state;
pub mod style;

pub fn router() -> topcoat::router::RouterBuilder {
    topcoat::router::module_router!()
}

#[topcoat::router::route(GET "/robots.txt")]
pub async fn robots_txt() -> TopcoatResult<&'static str> {
    Ok("User-agent: *\nDisallow: /profile/\n")
}

#[topcoat::router::layer]
pub async fn csp_layer(cx: &Cx, body: Body, next: Next<'_>) -> TopcoatResult<Response> {
    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        topcoat::router::header::HeaderName::from_static("content-security-policy"),
        topcoat::router::header::HeaderValue::from_static(
            "default-src 'self'; frame-ancestors 'none'; form-action 'self'",
        ),
    );
    headers.append(
        topcoat::router::header::HeaderName::from_static("referrer-policy"),
        topcoat::router::header::HeaderValue::from_static("same-origin"),
    );
    next.run(cx, body).await
}

use crate::app::components::button::{ButtonVariant, button_link};
use crate::app::navigation::{NavigationLayout, classify_route, resolve_safe_back_url};

#[topcoat::router::layout]
pub async fn root_layout(
    cx: &Cx,
    slot: topcoat::router::Slot<'_>,
) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::app::auth::guard::current_user(cx).await?;
    let path = topcoat::router::request::uri(cx).path();
    let user_handle = user.as_ref().map(|u| u.handle.as_str());
    let nav_layout = classify_route(path, user_handle);

    let site_logo_href = if user.is_some() { "/offers" } else { "/" };
    let (auth_link_href, auth_link_text) = if user.is_some() {
        ("/profile", "Profile")
    } else {
        ("/auth/sign-in", "Sign In")
    };

    let (is_child, back_url) = match nav_layout {
        NavigationLayout::Main => (false, String::new()),
        NavigationLayout::Child { fallback_parent } => {
            let url = resolve_safe_back_url(
                topcoat::router::request::headers(cx),
                path,
                fallback_parent,
            );
            (true, url)
        }
    };

    Ok(view! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Haven"</title>
                <link rel="stylesheet" href="/style.css" />
            </head>
            <body class=(if is_child { "layout-child" } else { "layout-main" })>
                if is_child {
                    <header class="site-header desktop-only">
                        <nav class="nav-container">
                            <a href=(site_logo_href) class="site-logo">"Haven"</a>
                            <ul class="nav-links">
                                <li><a href="/offers">"Offers"</a></li>
                                <li><a href=(auth_link_href)>(auth_link_text)</a></li>
                            </ul>
                        </nav>
                    </header>
                    <header class="child-header">
                        <div class="child-header-container">
                            <a href=(back_url) class="child-back-btn" aria-label="Go back">
                                <svg class="back-icon-svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                    <path d="M19 12H5" />
                                    <path d="M12 19l-7-7 7-7" />
                                </svg>
                                <span class="back-text">"Back"</span>
                            </a>
                        </div>
                    </header>
                } else {
                    <header class="site-header">
                        <nav class="nav-container">
                            <a href=(site_logo_href) class="site-logo">"Haven"</a>
                            <ul class="nav-links">
                                <li><a href="/offers">"Offers"</a></li>
                                <li><a href=(auth_link_href)>(auth_link_text)</a></li>
                            </ul>
                        </nav>
                    </header>
                }
                <main class="main-container">
                    (slot)
                </main>
            </body>
        </html>
    })
}

#[topcoat::router::page]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    if let Some(_user) = crate::app::auth::guard::current_user(cx).await? {
        return Err(topcoat::router::error::redirect("/offers").into());
    }

    Ok(view! {
        <div class="landing">
            <h1 class="landing-title">"Exchange offers directly"</h1>
            <p class="landing-lead">
                "Put something you are offering on Haven, or browse what others have made available."
            </p>
            <div class="landing-actions">
                button_link(
                    href: "/offers",
                    text: "Browse Offers",
                    variant: ButtonVariant::Primary,
                )
                button_link(
                    href: "/auth/sign-in",
                    text: "Sign In",
                    variant: ButtonVariant::Secondary,
                )
            </div>
        </div>
    })
}
