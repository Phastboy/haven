#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::see_other,
    Result as TopcoatResult,
};

#[derive(serde::Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

#[topcoat::router::page]
pub async fn verify_prompt(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let query = topcoat::router::parse_query_params::<VerifyQuery>(cx)?;
    let token = query.token;
    Ok(topcoat::view::view! {
        ( (topcoat::router::header::REFERRER_POLICY, topcoat::router::HeaderValue::from_static("no-referrer")) )
        <div class="verify">
            <h1>"Sign In Verification"</h1>
            <form method="post" action="/auth/verify">
                <input type="hidden" name="token" value=(token) />
                <button type="submit">"Click here to sign in"</button>
            </form>
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct VerifyForm {
    pub token: String,
}

#[topcoat::router::page(POST)]
pub async fn verify_submit(
    cx: &Cx,
    form: topcoat::router::content::Form<VerifyForm>,
) -> TopcoatResult<()> {
    let raw_token = form.0.token;
    let hashed = haven_domain::session::hash_token(&raw_token);
    let db = crate::cx_helpers::db(cx);
    
    let magic_link = haven_db::magic_links::find_by_token_hash(db, &hashed)
        .await
        .map_err(topcoat::Error::from)?
        .ok_or_else(|| topcoat::router::error::not_found())?;
        
    if magic_link.used_at.is_some() || chrono::Utc::now() > magic_link.expires_at {
        return Err(topcoat::router::error::unauthorized().into());
    }
    
    haven_db::magic_links::consume(db, magic_link.id).await.map_err(topcoat::Error::from)?;
    
    let account = haven_db::accounts::find_by_email(db, &magic_link.email)
        .await
        .map_err(topcoat::Error::from)?
        .ok_or_else(|| topcoat::router::error::not_found())?;
        
    let session = topcoat::session::start(cx).await?;
    let hash_hex = crate::cx_helpers::token_hash_hex(&session.token_hash);
    let hashed_token = haven_domain::session::HashedToken::from_hex(hash_hex);
    let expires_at = chrono::DateTime::<chrono::Utc>::from(session.expires_at);
    
    let ip = topcoat::router::request::client_ip(cx).map(|ip| ip.to_string());
    let user_agent = topcoat::router::request::headers(cx)
        .get("user-agent")
        .and_then(|h| h.to_str().ok().map(|s| s.to_string()));
    
    haven_db::sessions::create(db, account.id, &hashed_token, expires_at, ip.as_deref(), user_agent.as_deref())
        .await
        .map_err(topcoat::Error::from)?;
        
    // Ensure the User record exists (created on first sign-in)
    haven_db::users::find_or_create(db, account.id).await.map_err(topcoat::Error::from)?;
        
    Err(see_other("/offers").into())
}
