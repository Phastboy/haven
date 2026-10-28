//! Shared fixed-vocabulary topic control for publishing and discovery.

use haven_domain::offer::Category;
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Renders all launch topics in stable order, retaining invalid submitted input.
#[component]
pub async fn category_select(
    #[into] id: String,
    #[default] selected: Option<String>,
    #[default] required: bool,
    #[default] error: Option<String>,
) -> TopcoatResult<impl View> {
    let selected = selected.unwrap_or_default();
    let unknown = !selected.is_empty() && Category::parse(&selected).is_err();
    let error_id = format!("{id}-error");
    Ok(view! {
        <div class="filter-field category-field">
            <label for=(id.clone()) class="field-label">"Category"</label>
            <select id=(id) name="category" class="field-input field-select"
                required=(required.then_some("required"))
                aria-invalid=(error.is_some().then_some("true"))
                aria-describedby=(error.as_ref().map(|_| error_id.clone()))>
                <option value="" selected=(selected.is_empty().then_some("selected"))>
                    (if required { "Choose a category" } else { "All categories" })
                </option>
                if unknown {
                    <option value=(selected.clone()) selected="selected">(format!("Unknown category: {selected}"))</option>
                }
                for category in Category::ALL {
                    <option value=(category.as_str()) selected=((selected == category.as_str()).then_some("selected"))>(category.label())</option>
                }
            </select>
            if let Some(error) = error {
                <p id=(error_id) class="field-error" role="alert">(error)</p>
            }
        </div>
    })
}
