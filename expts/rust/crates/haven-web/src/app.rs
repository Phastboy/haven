#![allow(
    clippy::too_many_lines,
    reason = "Root layout with desktop and mobile nav shells and unread badge"
)]

use haven_domain::offer::{OfferFilter, PublicFeedQuery};
use topcoat::asset::{AssetBundle, RouterBuilderAssetExt};
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
#[cfg(test)]
mod layout_tests;
pub mod navigation;
pub mod offers;
pub mod origin;
pub mod profile;
pub mod profiles;
pub mod state;
pub mod style;

pub fn router() -> std::io::Result<topcoat::router::RouterBuilder> {
    let assets = load_asset_bundle()?;
    Ok(topcoat::router::module_router!().assets(assets))
}

fn load_asset_bundle() -> std::io::Result<AssetBundle> {
    if let Ok(bundle) = AssetBundle::load() {
        return Ok(bundle);
    }

    // Cargo test executables live in target/{profile}/deps, while Topcoat
    // writes the shared bundle to target/{profile}/assets.
    let executable = std::env::current_exe()?;
    let executable_dir = executable
        .parent()
        .ok_or_else(|| std::io::Error::other("executable has no parent directory"))?;
    let bundle_dir = if executable_dir
        .file_name()
        .is_some_and(|name| name == "deps")
    {
        executable_dir.parent().unwrap_or(executable_dir)
    } else {
        executable_dir
    };
    AssetBundle::load_dir(bundle_dir.join("assets"))
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

use crate::app::components::{
    landing_hero::landing_hero,
    nav::{
        RouteKind, ViewerKind,
        app_shell::{AppShellData, app_shell},
    },
    recent_offers::{RecentOffersData, recent_offers},
};
use crate::app::navigation::{NavigationLayout, classify_route, resolve_safe_back_url};

#[topcoat::router::layout]
pub async fn root_layout(
    cx: &Cx,
    slot: topcoat::router::Slot<'_>,
) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::app::auth::guard::current_user(cx).await?;
    let path = topcoat::router::request::uri(cx).path();
    let is_offer_detail = path
        .strip_prefix("/offers/")
        .is_some_and(|tail| !tail.is_empty() && !tail.contains('/'));
    let user_handle = user.as_ref().map(|u| u.handle.as_str());
    let nav_layout = classify_route(path, user_handle);

    let has_unread = if let Some(ref u) = user {
        crate::app::state::registry(cx)
            .conversations()
            .unread_exists(u.id)
            .await
            .unwrap_or(false)
    } else {
        false
    };

    let (kind, back_url) = match nav_layout {
        NavigationLayout::Main => (RouteKind::Main, String::new()),
        NavigationLayout::Child { fallback_parent } => {
            let url =
                resolve_safe_back_url(topcoat::router::request::headers(cx), path, fallback_parent);
            (RouteKind::Child, url)
        }
    };
    let viewer = if user.is_some() {
        ViewerKind::SignedIn
    } else {
        ViewerKind::Visitor
    };
    let auth_page = path.starts_with("/auth/");

    Ok(view! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Haven"</title>
                <link rel="stylesheet" href="/style.css" />
                if !is_offer_detail {
                    <script src="/cover-images.js" defer="defer"></script>
                    topcoat::runtime::script()
                }
            </head>
            <body>
                app_shell(props: AppShellData {
                    viewer,
                    route: path,
                    kind,
                    has_unread,
                    back_href: back_url,
                    auth_page,
                },
                (slot)
            )
            </body>
        </html>
    })
}

#[topcoat::router::page]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    if let Some(_user) = crate::app::auth::guard::current_user(cx).await? {
        return Err(topcoat::router::error::redirect("/offers").into());
    }

    let query = PublicFeedQuery::new(OfferFilter::default(), None, 8);
    let page = crate::app::state::registry(cx)
        .offers()
        .list_public(&query)
        .await
        .map_err(crate::app::state::map_repo_err)?;
    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let cards = page
        .items
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            let cover = item.offer.cover_image();
            let (cover_url, cover_srcset) = cover.map_or((None, None), |image| {
                (
                    Some(crate::app::components::image::card_sq_url(
                        cloud_name,
                        image.public_id.as_str(),
                    )),
                    Some(crate::app::components::image::card_sq_srcset(
                        cloud_name,
                        image.public_id.as_str(),
                    )),
                )
            });
            crate::app::components::listing_card::ListingCardData {
                title: item.offer.title.clone(),
                href: format!("/offers/{}", item.offer.slug),
                price_text: if item.offer.price.as_i32() == 0 {
                    "Free".into()
                } else {
                    format!(
                        "{} {}",
                        item.offer.price.as_i32(),
                        item.offer.currency.as_str()
                    )
                },
                maker_name: item.maker_display_name.as_str().to_owned(),
                maker_href: format!("/profiles/{}", item.maker_handle.as_str()),
                cover_url,
                cover_srcset,
                is_saved: false,
                return_to: "/".into(),
                is_authenticated: false,
                is_above_fold: idx < 2,
                slug: item.offer.slug,
            }
        })
        .collect();

    Ok(view! {
        <div class="landing-page">
            landing_hero()
            recent_offers(props: RecentOffersData { offers: cards })
        </div>
    })
}
