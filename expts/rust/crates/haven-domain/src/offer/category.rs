//! Fixed launch topics shared by publishing and discovery.

use serde::{Deserialize, Serialize};

/// One topic per offer. Slugs become permanent when Haven launches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Category {
    /// Fashion & beauty.
    FashionBeauty,
    /// Electronics.
    Electronics,
    /// Home & furniture.
    HomeFurniture,
    /// Housing & real estate.
    HousingRealEstate,
    /// Coaching & consultation.
    CoachingConsultation,
    /// Repairs & home services.
    RepairsHomeServices,
    /// Tools & equipment.
    ToolsEquipment,
    /// Books & learning.
    BooksLearning,
    /// Food & groceries.
    FoodGroceries,
    /// Stays & accommodation.
    StaysAccommodation,
    /// Other.
    Other,
}

impl Category {
    /// Launch topics in a stable order, including topics without public offers.
    pub const ALL: [Self; 11] = [
        Self::FashionBeauty,
        Self::Electronics,
        Self::HomeFurniture,
        Self::HousingRealEstate,
        Self::CoachingConsultation,
        Self::RepairsHomeServices,
        Self::ToolsEquipment,
        Self::BooksLearning,
        Self::FoodGroceries,
        Self::StaysAccommodation,
        Self::Other,
    ];

    /// Parses an exact stable slug; blank or unknown values are invalid.
    ///
    /// # Errors
    /// Returns [`CategoryError`] for an unrecognised slug.
    pub fn parse(raw: &str) -> Result<Self, CategoryError> {
        match raw {
            "fashion-beauty" => Ok(Self::FashionBeauty),
            "electronics" => Ok(Self::Electronics),
            "home-furniture" => Ok(Self::HomeFurniture),
            "housing-real-estate" => Ok(Self::HousingRealEstate),
            "coaching-consultation" => Ok(Self::CoachingConsultation),
            "repairs-home-services" => Ok(Self::RepairsHomeServices),
            "tools-equipment" => Ok(Self::ToolsEquipment),
            "books-learning" => Ok(Self::BooksLearning),
            "food-groceries" => Ok(Self::FoodGroceries),
            "stays-accommodation" => Ok(Self::StaysAccommodation),
            "other" => Ok(Self::Other),
            _ => Err(CategoryError),
        }
    }

    /// Stable URL and storage value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FashionBeauty => "fashion-beauty",
            Self::Electronics => "electronics",
            Self::HomeFurniture => "home-furniture",
            Self::HousingRealEstate => "housing-real-estate",
            Self::CoachingConsultation => "coaching-consultation",
            Self::RepairsHomeServices => "repairs-home-services",
            Self::ToolsEquipment => "tools-equipment",
            Self::BooksLearning => "books-learning",
            Self::FoodGroceries => "food-groceries",
            Self::StaysAccommodation => "stays-accommodation",
            Self::Other => "other",
        }
    }

    /// Human-readable topic name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FashionBeauty => "Fashion & beauty",
            Self::Electronics => "Electronics",
            Self::HomeFurniture => "Home & furniture",
            Self::HousingRealEstate => "Housing & real estate",
            Self::CoachingConsultation => "Coaching & consultation",
            Self::RepairsHomeServices => "Repairs & home services",
            Self::ToolsEquipment => "Tools & equipment",
            Self::BooksLearning => "Books & learning",
            Self::FoodGroceries => "Food & groceries",
            Self::StaysAccommodation => "Stays & accommodation",
            Self::Other => "Other",
        }
    }
}

/// An offer must use a topic from the launch vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Choose a valid category.")]
pub struct CategoryError;

impl TryFrom<String> for Category {
    type Error = CategoryError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Category> for String {
    fn from(value: Category) -> Self {
        value.as_str().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_slugs_round_trip_and_reject_unknown_values() {
        let mut slugs = std::collections::HashSet::new();
        for category in Category::ALL {
            assert_eq!(Category::parse(category.as_str()), Ok(category));
            assert!(slugs.insert(category.as_str()));
            assert_ne!(category.label(), "");
        }
        for invalid in ["", "food", "Food-groceries", "other "] {
            assert_eq!(Category::parse(invalid), Err(CategoryError));
        }
    }
}
