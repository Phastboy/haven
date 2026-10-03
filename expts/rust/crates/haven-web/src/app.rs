use topcoat::{
    context::Cx,
    router::{response::Response, Body, Next},
    view::view,
    Result as TopcoatResult,
};

pub mod auth;
pub mod offers;

pub fn router() -> topcoat::router::RouterBuilder {
    topcoat::router::module_router!()
}

#[topcoat::router::layer]
pub async fn csp_layer(cx: &Cx, body: Body, next: Next<'_>) -> TopcoatResult<Response> {
    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        topcoat::router::header::HeaderName::from_static("content-security-policy"),
        topcoat::router::header::HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'; form-action 'self'"),
    );
    headers.append(
        topcoat::router::header::HeaderName::from_static("referrer-policy"),
        topcoat::router::header::HeaderValue::from_static("no-referrer"),
    );
    next.run(cx, body).await
}

#[topcoat::router::layout]
pub async fn root_layout(slot: topcoat::router::Slot<'_>) -> TopcoatResult<impl topcoat::view::View> {
    Ok(view! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <title>"Haven"</title>
            </head>
            <body>
                <header>
                    <a href="/">"Haven"</a>
                </header>
                <main>
                    (slot)
                </main>
            </body>
        </html>
    })
}

#[topcoat::router::page]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    if let Some(_user) = crate::cx_helpers::current_user(cx).await? {
        return Err(topcoat::router::error::redirect("/offers").into());
    }

    Ok(view! {
        <div class="landing">
            <h1>"Welcome to Haven"</h1>
            <a href="/auth/sign-in">"Sign In"</a>
        </div>
    })
}
