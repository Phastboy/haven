#![allow(
    clippy::too_many_lines,
    reason = "Root layout with desktop and mobile nav shells and unread badge"
)]

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{Body, Next, response::Response},
    view::view,
};

pub mod auth;
pub mod broadcaster;
pub mod components;
pub mod conversations;
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
            "default-src 'self'; img-src 'self' https://res.cloudinary.com data:; frame-ancestors 'none'; form-action 'self'",
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

    let has_unread = if let Some(ref u) = user {
        crate::app::state::registry(cx)
            .conversations()
            .unread_exists(u.id)
            .await
            .unwrap_or(false)
    } else {
        false
    };

    let is_offers_active = path == "/" || path == "/offers";
    let is_profile_active = path == "/profile";
    let is_messages_active = path.starts_with("/conversations");

    let (is_child, back_url) = match nav_layout {
        NavigationLayout::Main => (false, String::new()),
        NavigationLayout::Child { fallback_parent } => {
            let url =
                resolve_safe_back_url(topcoat::router::request::headers(cx), path, fallback_parent);
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
                                if user.is_some() {
                                    <li>
                                        <a href="/conversations">
                                            "Messages"
                                            if has_unread {
                                                <span class="unread-badge" aria-label="Unread messages">"●"</span>
                                            }
                                        </a>
                                    </li>
                                }
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
                                if user.is_some() {
                                    <li>
                                        <a href="/conversations">
                                            "Messages"
                                            if has_unread {
                                                <span class="unread-badge" aria-label="Unread messages">"●"</span>
                                            }
                                        </a>
                                    </li>
                                }
                                <li><a href=(auth_link_href)>(auth_link_text)</a></li>
                            </ul>
                        </nav>
                    </header>
                }
                <main class="main-container">
                    (slot)
                </main>
                if !is_child {
                    <nav class="bottom-nav" aria-label="Mobile Navigation">
                        <div class="bottom-nav-container">
                            <a
                                href="/offers"
                                class=(if is_offers_active { "bottom-nav-link active" } else { "bottom-nav-link" })
                                aria-current=(if is_offers_active { Some("page") } else { None })
                            >
                                <svg class="bottom-nav-icon" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                    <rect x="3" y="3" width="7" height="7" />
                                    <rect x="14" y="3" width="7" height="7" />
                                    <rect x="14" y="14" width="7" height="7" />
                                    <rect x="3" y="14" width="7" height="7" />
                                </svg>
                                <span class="bottom-nav-text">"Offers"</span>
                            </a>
                            if user.is_some() {
                                <a
                                    href="/conversations"
                                    class=(if is_messages_active { "bottom-nav-link active" } else { "bottom-nav-link" })
                                    aria-current=(if is_messages_active { Some("page") } else { None })
                                >
                                    <svg class="bottom-nav-icon" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                                    </svg>
                                    <span class="bottom-nav-text">
                                        "Messages"
                                        if has_unread {
                                            <span class="unread-badge" aria-label="Unread messages">"●"</span>
                                        }
                                    </span>
                                </a>
                            }
                            <a
                                href=(auth_link_href)
                                class=(if is_profile_active { "bottom-nav-link active" } else { "bottom-nav-link" })
                                aria-current=(if is_profile_active { Some("page") } else { None })
                            >
                                <svg class="bottom-nav-icon" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                                    <circle cx="12" cy="7" r="4" />
                                </svg>
                                <span class="bottom-nav-text">(auth_link_text)</span>
                            </a>
                        </div>
                    </nav>
                }
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
