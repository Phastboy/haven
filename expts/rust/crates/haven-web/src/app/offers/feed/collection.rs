//! Session-resolved discovery scope, activity cursors and response privacy.

use super::{criteria::ParsedCriteria, parse_after_cursor};
use crate::app::components::{chip::ChipData, chip_row::ChipRowData};
use haven_domain::offer::{
    PersonalCursor, PersonalFeedQuery, PersonalScope, PublicFeedQuery, PublicOfferPage,
};
use haven_domain::user::User;
use topcoat::{Result as TopcoatResult, context::Cx};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Collection {
    Public,
    Saved,
    Inquired,
}

impl Collection {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "" | "public" => Some(Self::Public),
            "saved" => Some(Self::Saved),
            "inquired" => Some(Self::Inquired),
            _ => None,
        }
    }
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Saved => "saved",
            Self::Inquired => "inquired",
        }
    }
    pub(super) const fn heading(self) -> Option<&'static str> {
        match self {
            Self::Public => None,
            Self::Saved => Some("Your saved offers"),
            Self::Inquired => Some("Offers you inquired about"),
        }
    }
}

pub(super) struct FeedPage {
    pub(super) page: PublicOfferPage,
    pub(super) viewer: Option<User>,
    pub(super) return_after: Option<String>,
    pub(super) cursor_notice: bool,
}

pub(super) fn row(parsed: &ParsedCriteria) -> ChipRowData {
    let choices = [
        (Collection::Public, "All offers", "All available offers"),
        (Collection::Saved, "Saved", "Your saved offers"),
        (
            Collection::Inquired,
            "Inquired",
            "Offers you inquired about",
        ),
    ];
    ChipRowData {
        chips: choices
            .into_iter()
            .map(|(collection, label, name)| ChipData {
                label: label.to_string(),
                href: parsed.scope_href(collection),
                selected: Collection::parse(&parsed.controls.scope) == Some(collection),
                accessible_label: Some(name.to_string()),
            })
            .collect(),
        accessible_name: "Offer collection".to_string(),
    }
}

pub(super) async fn fetch(
    cx: &Cx,
    parsed: &ParsedCriteria,
    after: Option<&str>,
) -> TopcoatResult<FeedPage> {
    let viewer = crate::app::auth::guard::current_user(cx).await?;
    let collection = Collection::parse(&parsed.controls.scope);
    let personal = matches!(collection, Some(Collection::Saved | Collection::Inquired));
    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        topcoat::router::header::HeaderName::from_static("vary"),
        topcoat::router::header::HeaderValue::from_static("Cookie"),
    );
    if personal || viewer.is_some() {
        headers.append(
            topcoat::router::header::HeaderName::from_static("cache-control"),
            topcoat::router::header::HeaderValue::from_static("private, no-store"),
        );
    }
    if personal {
        headers.append(
            topcoat::router::header::HeaderName::from_static("x-robots-tag"),
            topcoat::router::header::HeaderValue::from_static("noindex, follow"),
        );
        if viewer.is_none() {
            let mut query = url::form_urlencoded::Serializer::new(String::new());
            query.append_pair("return_to", &parsed.href(None));
            return Err(topcoat::router::error::see_other(format!(
                "/auth/sign-in?{}",
                query.finish()
            ))
            .into());
        }
    }
    let empty = || PublicOfferPage {
        items: Vec::new(),
        next_cursor: None,
    };
    let invalid = parsed.controls.category_error.is_some() || collection.is_none();
    let repo = crate::app::state::registry(cx).offers();
    match collection {
        Some(Collection::Public) | None => {
            let (cursor, return_after) = parse_after_cursor(after);
            let page = if invalid {
                empty()
            } else {
                repo.list_public(&PublicFeedQuery::new(parsed.criteria.clone(), cursor, 20))
                    .await
                    .map_err(crate::app::state::map_repo_err)?
            };
            Ok(FeedPage {
                page,
                viewer,
                cursor_notice: after.is_some() && return_after.is_none(),
                return_after,
            })
        }
        Some(Collection::Saved | Collection::Inquired) => {
            let person = viewer.as_ref().ok_or_else(|| {
                topcoat::router::error::bad_request("Sign in to view your offers.")
            })?;
            let scope = match collection {
                Some(Collection::Saved) => PersonalScope::Saved(person.id),
                Some(Collection::Inquired) => PersonalScope::Inquired(person.id),
                Some(Collection::Public) | None => {
                    return Err(topcoat::router::error::bad_request("Invalid collection.").into());
                }
            };
            let cursor =
                after.and_then(|raw| PersonalCursor::decode(raw, scope, &parsed.criteria).ok());
            let return_after = cursor.as_ref().map(PersonalCursor::encode);
            let mut query = PersonalFeedQuery::new(scope, parsed.criteria.clone());
            if let Some(cursor) = cursor {
                query = query
                    .after(cursor)
                    .map_err(|_| topcoat::router::error::bad_request("Invalid page."))?;
            }
            let page = if invalid {
                empty()
            } else {
                repo.list_personal(&query)
                    .await
                    .map_err(crate::app::state::map_repo_err)?
            };
            Ok(FeedPage {
                page,
                viewer,
                cursor_notice: after.is_some() && return_after.is_none(),
                return_after,
            })
        }
    }
}
