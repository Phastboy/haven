//! Reusable zero-JavaScript confirmation dialog using HTML Popover and Dialog APIs.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Props for [`confirm_dialog`].
#[component]
pub async fn confirm_dialog(
    #[into] id: String,
    #[into] title: String,
    #[into] message: String,
    #[into] action_url: String,
    #[into] confirm_label: String,
    #[default] cancel_label: Option<String>,
    #[default] is_danger: bool,
) -> TopcoatResult<impl View> {
    let cancel_text = cancel_label.unwrap_or_else(|| "Cancel".to_string());
    let confirm_btn_class = if is_danger {
        "btn btn-danger"
    } else {
        "btn btn-primary"
    };

    Ok(view! {
        <dialog id=(id.clone()) popover="auto" class="confirm-dialog">
            <div class="confirm-dialog-content">
                <h2 class="dialog-title">(title)</h2>
                <p class="dialog-message">(message)</p>
                <div class="dialog-actions">
                    <button
                        type="button"
                        popovertarget=(id.clone())
                        popovertargetaction="hide"
                        class="btn btn-secondary"
                    >
                        (cancel_text)
                    </button>
                    <form method="post" action=(action_url) class="dialog-confirm-form">
                        <button type="submit" class=(confirm_btn_class)>
                            (confirm_label)
                        </button>
                    </form>
                </div>
            </div>
        </dialog>
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
    async fn test_confirm_dialog_renders_markup() {
        let cx = Cx::default();
        let __cx = &cx;
        let dlg = view! {
            confirm_dialog(
                id: "test-dialog",
                title: "Sign out?",
                message: "Are you sure you want to sign out of Haven?",
                action_url: "/auth/sign-out",
                confirm_label: "Sign out",
                is_danger: true,
            )
        };

        let html = dlg.single().await.unwrap().render(__cx);
        assert!(
            html.contains("<dialog id=\"test-dialog\" popover=\"auto\" class=\"confirm-dialog\">")
        );
        assert!(html.contains("Sign out?"));
        assert!(html.contains("Are you sure you want to sign out of Haven?"));
        assert!(html.contains("popovertarget=\"test-dialog\""));
        assert!(html.contains("popovertargetaction=\"hide\""));
        assert!(html.contains("action=\"/auth/sign-out\""));
        assert!(html.contains("method=\"post\""));
        assert!(html.contains("btn-danger"));
        assert!(html.contains("Sign out"));
        assert!(html.contains("Cancel"));
    }
}
