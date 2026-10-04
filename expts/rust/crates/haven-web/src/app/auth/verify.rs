
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
    let registry = crate::cx_helpers::registry(cx);
    let map_err = crate::cx_helpers::map_repo_err;
    
    let magic_link = registry.magic_links().find_by_token_hash(&hashed)
        .await
        .map_err(map_err)?
        .ok_or_else(topcoat::router::error::unauthorized)?;
        
    if magic_link.used_at.is_some() || chrono::Utc::now() > magic_link.expires_at {
        return Err(topcoat::router::error::unauthorized().into());
    }
    
    registry.magic_links().mark_used(magic_link.id).await.map_err(map_err)?;
    
    let account = registry.accounts().find_by_email(&magic_link.email)
        .await
        .map_err(map_err)?
        .ok_or_else(topcoat::router::error::not_found)?;
        
    let session = topcoat::session::start(cx).await?;
    let hash_hex = crate::cx_helpers::token_hash_hex(&session.token_hash);
    let hashed_token = haven_domain::session::HashedToken::from_hex(hash_hex);
    let expires_at = chrono::DateTime::<chrono::Utc>::from(session.expires_at);
    
    let ip = topcoat::router::request::client_ip(cx).map(|ip| ip.to_string());
    let user_agent = topcoat::router::request::headers(cx)
        .get("user-agent")
        .and_then(|h| h.to_str().ok().map(std::string::ToString::to_string));
    
    let ip_addr = ip.and_then(|s| s.parse::<std::net::IpAddr>().ok());
    registry.sessions().create(account.id, &hashed_token, expires_at, ip_addr, user_agent.as_deref())
        .await
        .map_err(map_err)?;
        
    // Ensure the User record exists (created on first sign-in)
    let user = registry.users().find_by_account(account.id).await.map_err(map_err)?;
    if user.is_none() {
        registry.users().create(account.id).await.map_err(map_err)?;
    }
        
    Err(see_other("/offers").into())
}
