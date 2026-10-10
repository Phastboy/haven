//! Public profile routes.

pub mod handle;

use topcoat::{Result as TopcoatResult, router::error::see_other};

/// Redirects `/profiles` to `/offers` to prevent dead-end browsing.
#[topcoat::router::page("/profiles")]
pub async fn index() -> TopcoatResult<impl topcoat::view::View> {
    if true {
        return Err(see_other("/offers").into());
    }
    Ok(topcoat::view::view! { "" })
}
