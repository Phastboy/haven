use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{Body, Next, response::Response},
    view::view,
};

pub mod auth;
pub mod components;
pub mod health;
pub mod offers;
pub mod state;
pub mod style;

pub fn router() -> topcoat::router::RouterBuilder {
    topcoat::router::module_router!()
}

#[topcoat::router::route(GET "/robots.txt")]
pub async fn robots_txt() -> TopcoatResult<&'static str> {
    Ok("User-agent: *\nDisallow: /offers/manage/\n")
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
        topcoat::router::header::HeaderValue::from_static("no-referrer"),
    );
    next.run(cx, body).await
}

use crate::app::components::button::{ButtonVariant, button_link};

#[topcoat::router::layout]
pub async fn root_layout(
    slot: topcoat::router::Slot<'_>,
) -> TopcoatResult<impl topcoat::view::View> {
    Ok(view! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Haven"</title>
                <link rel="stylesheet" href="/style.css" />
            </head>
            <body>
                <header class="site-header">
                    <nav class="nav-container">
                        <a href="/" class="site-logo">"Haven"</a>
                        <ul class="nav-links">
                            <li><a href="/offers">"Offers"</a></li>
                        </ul>
                    </nav>
                </header>
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
