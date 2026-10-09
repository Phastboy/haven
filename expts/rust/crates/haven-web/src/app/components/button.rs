use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Visual variants for buttons adhering to Haven monochrome-first design tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
}

impl ButtonVariant {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Primary => "btn btn-primary",
            Self::Secondary => "btn btn-secondary",
            Self::Danger => "btn btn-danger",
        }
    }
}

/// A styled button element.
#[component]
pub async fn button(
    #[into] text: String,
    #[default] variant: ButtonVariant,
    #[default("submit")] button_type: &'static str,
    #[default] disabled: bool,
) -> TopcoatResult<impl View> {
    let class = variant.class_name();
    Ok(view! {
        <button
            type=(button_type)
            class=(class)
            disabled=(disabled.then_some("disabled"))
        >
            (text)
        </button>
    })
}

/// A link styled as a button.
#[component]
pub async fn button_link(
    #[into] href: String,
    #[into] text: String,
    #[default] variant: ButtonVariant,
) -> TopcoatResult<impl View> {
    let class = variant.class_name();
    Ok(view! {
        <a href=(href) class=(class)>
            (text)
        </a>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn button_variants_render_expected_classes() {
        let cx = Cx::default();
        let __cx = &cx;

        let primary = view! {
            button(
                text: "Submit",
                variant: ButtonVariant::Primary,
            )
        };
        let html_primary = primary.single().await.unwrap().render(__cx);
        assert!(html_primary.contains("class=\"btn btn-primary\""));
        assert!(html_primary.contains("Submit"));

        let secondary = view! {
            button(
                text: "Cancel",
                variant: ButtonVariant::Secondary,
            )
        };
        let html_sec = secondary.single().await.unwrap().render(__cx);
        assert!(html_sec.contains("class=\"btn btn-secondary\""));

        let danger = view! {
            button(
                text: "Delete",
                variant: ButtonVariant::Danger,
            )
        };
        let html_danger = danger.single().await.unwrap().render(__cx);
        assert!(html_danger.contains("class=\"btn btn-danger\""));

        let disabled_btn = view! {
            button(
                text: "Disabled",
                variant: ButtonVariant::Secondary,
                disabled: true,
            )
        };
        let html_disabled = disabled_btn.single().await.unwrap().render(__cx);
        assert!(html_disabled.contains("disabled=\"disabled\""));

        let link = view! {
            button_link(
                href: "/cancel",
                text: "Back",
                variant: ButtonVariant::Secondary,
            )
        };
        let html_link = link.single().await.unwrap().render(__cx);
        assert!(html_link.contains("href=\"/cancel\""));
        assert!(html_link.contains("class=\"btn btn-secondary\""));
    }
}
