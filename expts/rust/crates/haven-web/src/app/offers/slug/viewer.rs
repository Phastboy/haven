//! Who is looking at a public offer page, and how they may act.

use haven_domain::user::UserId;

/// Role of the person requesting `/offers/{slug}`.
///
/// Computed once from the session and the offer owner so each UI state is a
/// single match arm rather than scattered ownership checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfferViewer {
    /// No session. Markup must be identical for every anonymous request so
    /// public `Cache-Control` and `ETag` responses stay shareable.
    Anonymous,
    /// Signed-in person who does not own the offer.
    Visitor {
        /// Whether this person has already saved the offer.
        saved: bool,
    },
    /// The offer's maker previewing their own public (or private) page.
    Owner,
}

/// Inputs for [`offer_viewer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfferViewerInput {
    /// Authenticated viewer's id, if any.
    pub viewer_id: Option<UserId>,
    /// Offer maker's id.
    pub owner_id: UserId,
    /// Whether the viewer has saved this offer (ignored for owner / anonymous).
    pub saved: bool,
}

/// Classifies the request into an [`OfferViewer`] role.
#[must_use]
pub fn offer_viewer(input: OfferViewerInput) -> OfferViewer {
    match input.viewer_id {
        None => OfferViewer::Anonymous,
        Some(id) if id == input.owner_id => OfferViewer::Owner,
        Some(_) => OfferViewer::Visitor { saved: input.saved },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_anonymous_visitor_and_owner() {
        let owner = UserId::generate();
        let other = UserId::generate();

        assert_eq!(
            offer_viewer(OfferViewerInput {
                viewer_id: None,
                owner_id: owner,
                saved: true,
            }),
            OfferViewer::Anonymous
        );
        assert_eq!(
            offer_viewer(OfferViewerInput {
                viewer_id: Some(other),
                owner_id: owner,
                saved: false,
            }),
            OfferViewer::Visitor { saved: false }
        );
        assert_eq!(
            offer_viewer(OfferViewerInput {
                viewer_id: Some(other),
                owner_id: owner,
                saved: true,
            }),
            OfferViewer::Visitor { saved: true }
        );
        assert_eq!(
            offer_viewer(OfferViewerInput {
                viewer_id: Some(owner),
                owner_id: owner,
                saved: false,
            }),
            OfferViewer::Owner
        );
    }
}
