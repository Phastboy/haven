use topcoat::{Result as TopcoatResult, context::Cx};

#[topcoat::router::page]
pub async fn sent_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let _ = cx;
    Ok(topcoat::view::view! {
        <div class="sent-page">
            <h1>"Check your email"</h1>
            <p>"If an account exists for that email, we've sent a magic link."</p>
        </div>
    })
}
