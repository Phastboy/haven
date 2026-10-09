use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Summary banner rendering form-level validation or submission error.
#[component]
pub async fn form_error(#[into] message: String) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="form-error-summary" role="alert">
            (message)
        </div>
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
    async fn form_error_renders_alert_role_and_message() {
        let cx = Cx::default();
        let __cx = &cx;
        let el = view! {
            form_error(message: "Invalid credentials provided")
        };
        let html = el.single().await.unwrap().render(__cx);
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("class=\"form-error-summary\""));
        assert!(html.contains("Invalid credentials provided"));
    }
}
