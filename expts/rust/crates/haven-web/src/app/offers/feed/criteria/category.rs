//! Topic validation, chip presentation and stable URL contribution.

use haven_domain::offer::Category;

use super::ParsedCriteria;
use crate::app::components::{chip::ChipData, chip_row::ChipRowData};

pub(super) fn parse(raw: &str) -> (Option<Category>, Option<String>) {
    if raw.is_empty() {
        return (None, None);
    }
    match Category::parse(raw) {
        Ok(category) => (Some(category), None),
        Err(error) => (None, Some(error.to_string())),
    }
}

pub(super) fn append_url(serializer: &mut url::form_urlencoded::Serializer<'_, String>, raw: &str) {
    if !raw.is_empty() {
        serializer.append_pair("category", raw);
    }
}

/// Build display data here so generic chip components never depend on topics.
pub(super) fn row(parsed: &ParsedCriteria) -> ChipRowData {
    let all = ChipData {
        label: "All".to_string(),
        href: parsed.category_href(None),
        selected: parsed.controls.category.is_empty(),
        accessible_label: Some("All categories".to_string()),
    };
    // Browse order follows the reference; publishing retains the full vocabulary.
    let topics = [
        Category::FoodGroceries,
        Category::StaysAccommodation,
        Category::FashionBeauty,
        Category::Electronics,
        Category::HomeFurniture,
        Category::HousingRealEstate,
        Category::CoachingConsultation,
        Category::RepairsHomeServices,
        Category::ToolsEquipment,
        Category::BooksLearning,
        Category::Other,
    ];
    let chips = std::iter::once(all)
        .chain(topics.into_iter().map(|category| ChipData {
            label: chip_label(category).to_string(),
            href: parsed.category_href(Some(category)),
            selected: parsed.criteria.category == Some(category),
            accessible_label: Some(category.label().to_string()),
        }))
        .collect();
    ChipRowData {
        chips,
        accessible_name: "Offer categories".to_string(),
    }
}

fn chip_label(category: Category) -> &'static str {
    match category {
        Category::FoodGroceries => "Food",
        Category::StaysAccommodation => "Stays",
        Category::FashionBeauty => "Fashion & beauty",
        Category::Electronics => "Electronics",
        Category::HomeFurniture => "Home goods",
        Category::HousingRealEstate => "Housing",
        Category::CoachingConsultation => "Coaching",
        Category::RepairsHomeServices => "Repairs",
        Category::ToolsEquipment => "Tools",
        Category::BooksLearning => "Learning",
        Category::Other => "Other",
    }
}
