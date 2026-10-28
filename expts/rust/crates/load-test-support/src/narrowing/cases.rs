//! Expected page IDs are filtered in memory from fixture truth, independently of SQL predicates.
use super::{ActivityPosition, Actor, Case, TruthRow};
use haven_domain::offer::{
    Category, CurrencyCode, OfferCursor, OfferId, PersonalCursor, Price, PriceFilter,
    PublicCriteria, SearchQuery,
};

struct Facets {
    name: &'static str,
    category: Option<Category>,
    combined: bool,
    free: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "Independent facet and page cases in the fixture oracle"
)]
pub(super) fn for_actor(actor: &Actor, rows: &[TruthRow]) -> Vec<Case> {
    let definitions = [
        Facets {
            name: "all",
            category: None,
            combined: false,
            free: false,
        },
        Facets {
            name: "dominant",
            category: Some(Category::FoodGroceries),
            combined: false,
            free: false,
        },
        Facets {
            name: "rare",
            category: Some(Category::ToolsEquipment),
            combined: false,
            free: false,
        },
        Facets {
            name: "empty",
            category: Some(Category::StaysAccommodation),
            combined: false,
            free: false,
        },
        Facets {
            name: "combined",
            category: Some(Category::FoodGroceries),
            combined: true,
            free: false,
        },
        Facets {
            name: "free",
            category: Some(Category::FoodGroceries),
            combined: false,
            free: true,
        },
    ];
    let mut cases = Vec::new();
    for facets in definitions {
        let criteria = PublicCriteria {
            category: facets.category,
            title: facets
                .combined
                .then(|| SearchQuery::parse("Test Offer").unwrap()),
            price: if facets.free {
                Some(PriceFilter::free(CurrencyCode::Ngn))
            } else if facets.combined {
                Some(
                    PriceFilter::new(
                        CurrencyCode::Ngn,
                        Some(Price::new(1001).unwrap()),
                        Some(Price::new(1005).unwrap()),
                    )
                    .unwrap(),
                )
            } else {
                None
            },
        };
        let mut matches: Vec<_> = rows
            .iter()
            .filter_map(|row| {
                if row.image_count < 1
                    || facets
                        .category
                        .is_some_and(|category| category.as_str() != row.category)
                    || (facets.free && (row.price != 0 || row.currency != "NGN"))
                    || (facets.combined
                        && (row.currency != "NGN"
                            || row.price < 1001
                            || row.price > 1005
                            || !row.title.starts_with("Test Offer")))
                {
                    return None;
                }
                let position = if let Some(positions) = actor.positions.as_ref() {
                    *positions.get(&row.id)?
                } else {
                    ActivityPosition {
                        at: row.created_at,
                        id: row.id,
                    }
                };
                Some((row, position))
            })
            .collect();
        matches.sort_by_key(|row| std::cmp::Reverse((row.1.at, row.1.id)));
        let class = actor.scope.map_or("public", |scope| scope.label());
        for page in 0..2 {
            if page == 1 && matches.is_empty() {
                continue;
            }
            let offset = if page == 0 { 0 } else { matches.len().min(20) };
            let mut query = url::form_urlencoded::Serializer::new(String::new());
            if class != "public" {
                query.append_pair("scope", class);
            }
            if let Some(category) = facets.category {
                query.append_pair("category", category.as_str());
            }
            if facets.combined {
                query.append_pair("q", "Test Offer");
            }
            if facets.free || facets.combined {
                query.append_pair("currency", "NGN");
            }
            if facets.combined {
                query.append_pair("min_price", "1001");
                query.append_pair("max_price", "1005");
            }
            if facets.free {
                query.append_pair("max_price", "0");
            }
            if page == 1 {
                let boundary = matches.get(offset - 1).unwrap().1;
                let cursor = if let Some(scope) = actor.scope {
                    PersonalCursor::new(scope, &criteria, boundary).encode()
                } else {
                    OfferCursor::new(boundary.at, OfferId::from_uuid(boundary.id)).encode()
                };
                query.append_pair("after", &cursor);
            }
            let query = query.finish();
            let page_name = if page == 0 { "first" } else { "later" };
            cases.push(Case {
                id: format!(
                    "{class}-{}-{}-{}-{page_name}",
                    actor.history, actor.number, facets.name
                ),
                label: format!("{class}/{}/{}/{page_name}", actor.history, facets.name),
                class,
                history: actor.history,
                path: if query.is_empty() {
                    "/offers".to_string()
                } else {
                    format!("/offers?{query}")
                },
                session_token: actor.token.clone(),
                expected_slugs: matches
                    .iter()
                    .skip(offset)
                    .take(20)
                    .map(|(row, _)| row.slug.clone())
                    .collect(),
                expected_next: matches.len() > offset + 20,
                canonical: class == "public" && page == 0 && !facets.combined && !facets.free,
            });
        }
    }
    cases
}
