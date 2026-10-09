use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// A labeled text/number/email input field with optional help text and inline error.
#[component]
pub async fn text_field(
    #[into] label: String,
    #[into] name: String,
    #[default("text")] field_type: &'static str,
    #[default] value: Option<String>,
    #[default] placeholder: Option<String>,
    #[default] help: Option<String>,
    #[default] error: Option<String>,
    #[default] required: bool,
    #[default] minlength: Option<usize>,
    #[default] maxlength: Option<usize>,
    #[default] min: Option<i32>,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="field">
            <label class="field-label" for=(name.clone())>
                (label)
            </label>
            <input
                id=(name.clone())
                class="field-input"
                type=(field_type)
                name=(name)
                value=(value.unwrap_or_default())
                placeholder=(placeholder.unwrap_or_default())
                required=(required.then_some("required"))
                minlength=(minlength.map(|m| m.to_string()))
                maxlength=(maxlength.map(|m| m.to_string()))
                min=(min.map(|m| m.to_string()))
            />
            if let Some(err) = error {
                <span class="field-error">(err)</span>
            } else if let Some(h) = help {
                <span class="field-help">(h)</span>
            }
        </div>
    })
}

/// A labeled textarea field with optional help text and inline error.
#[component]
pub async fn textarea_field(
    #[into] label: String,
    #[into] name: String,
    #[default] value: Option<String>,
    #[default] placeholder: Option<String>,
    #[default] help: Option<String>,
    #[default] error: Option<String>,
    #[default] required: bool,
    #[default] minlength: Option<usize>,
    #[default] maxlength: Option<usize>,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="field">
            <label class="field-label" for=(name.clone())>
                (label)
            </label>
            <textarea
                id=(name.clone())
                class="field-textarea"
                name=(name)
                placeholder=(placeholder.unwrap_or_default())
                required=(required.then_some("required"))
                minlength=(minlength.map(|m| m.to_string()))
                maxlength=(maxlength.map(|m| m.to_string()))
            >
                (value.unwrap_or_default())
            </textarea>
            if let Some(err) = error {
                <span class="field-error">(err)</span>
            } else if let Some(h) = help {
                <span class="field-help">(h)</span>
            }
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
    async fn text_field_renders_label_and_error() {
        let cx = Cx::default();
        let __cx = &cx;
        let field = view! {
            text_field(
                label: "Offer Title",
                name: "title",
                value: Some("Vintage Lamp".to_string()),
                error: Some("Title too short".to_string()),
                required: true,
                minlength: Some(3),
            )
        };

        let html = field.single().await.unwrap().render(__cx);
        assert!(html.contains("for=\"title\""));
        assert!(html.contains("Offer Title"));
        assert!(html.contains("value=\"Vintage Lamp\""));
        assert!(html.contains("class=\"field-error\""));
        assert!(html.contains("Title too short"));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn textarea_field_renders_help_when_no_error() {
        let cx = Cx::default();
        let __cx = &cx;
        let field = view! {
            textarea_field(
                label: "Description",
                name: "description",
                value: Some("A nice lamp".to_string()),
                help: Some("Describe condition and pickup location".to_string()),
                required: true,
            )
        };

        let html = field.single().await.unwrap().render(__cx);
        assert!(html.contains("for=\"description\""));
        assert!(html.contains("A nice lamp"));
        assert!(html.contains("class=\"field-help\""));
        assert!(html.contains("Describe condition and pickup location"));
    }
}
