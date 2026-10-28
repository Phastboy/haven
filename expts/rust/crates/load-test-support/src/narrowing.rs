//! Pessimistic v0.7 fixtures and independent expected-result cases.
mod cases;
mod history;

use super::SessionOutput;
use chrono::{DateTime, Utc};
use haven_domain::offer::{ActivityPosition, PersonalScope};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct TruthRow {
    id: Uuid,
    user_id: Uuid,
    slug: String,
    title: String,
    category: String,
    price: i32,
    currency: String,
    image_count: i16,
    created_at: DateTime<Utc>,
}
struct Actor {
    scope: Option<PersonalScope>,
    history: &'static str,
    token: Option<String>,
    positions: Option<HashMap<Uuid, ActivityPosition>>,
    number: usize,
}
#[derive(Serialize)]
struct Case {
    id: String,
    label: String,
    class: &'static str,
    history: &'static str,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_token: Option<String>,
    expected_slugs: Vec<String>,
    expected_next: bool,
    canonical: bool,
}
#[derive(Serialize)]
struct Fixture {
    people: usize,
    offers: usize,
    category_counts: BTreeMap<String, usize>,
    cases: Vec<Case>,
}

pub(super) fn category(number: usize) -> haven_domain::offer::Category {
    use haven_domain::offer::Category;
    match number % 1000 {
        0..=899 => Category::FoodGroceries,
        900..=949 => Category::HomeFurniture,
        950..=979 => Category::Electronics,
        980..=989 => Category::FashionBeauty,
        990..=992 => Category::RepairsHomeServices,
        993..=994 => Category::BooksLearning,
        995 => Category::ToolsEquipment,
        996 => Category::CoachingConsultation,
        997 => Category::HousingRealEstate,
        _ => Category::Other,
    }
}

pub(super) async fn generate(
    pool: &PgPool,
    sessions: &[SessionOutput],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if sessions.len() < 10000 {
        return Err("narrowing fixtures require 10,000 people".into());
    }
    // A known historical offer timeline makes activity time independent from publication time.
    sqlx::query("UPDATE offer o SET created_at='2026-01-01'::timestamptz + t.n::int * interval '1 second', updated_at='2026-01-01'::timestamptz + t.n::int * interval '1 second' FROM (SELECT id,row_number() OVER(ORDER BY id) n FROM offer) t WHERE o.id=t.id").execute(pool).await?;
    let rows = sqlx::query_as::<_,TruthRow>("SELECT id,user_id,slug,title,category,price,currency,image_count,created_at FROM offer ORDER BY created_at DESC,id DESC").fetch_all(pool).await?;
    let mut counts = BTreeMap::new();
    for row in &rows {
        if row.image_count >= 1 {
            *counts.entry(row.category.clone()).or_insert(0) += 1;
        }
    }
    counts.entry("stays-accommodation".to_string()).or_insert(0);
    let public = Actor {
        scope: None,
        history: "none",
        token: None,
        positions: None,
        number: 0,
    };
    let mut cases = cases::for_actor(&public, &rows);
    for (number, session) in sessions.iter().take(17).enumerate() {
        let person = haven_domain::user::UserId::from_uuid(Uuid::parse_str(&session.user_id)?);
        let dense = number < 8;
        let count = if dense {
            10000
        } else if number < 16 {
            5
        } else {
            0
        };
        let history = if dense { "dense" } else { "sparse" };
        let (saved, inquired) =
            history::seed(pool, &rows, history::HistorySeed { person, count }).await?;
        for (scope, positions) in [
            (PersonalScope::Saved(person), saved),
            (PersonalScope::Inquired(person), inquired),
        ] {
            let actor = Actor {
                scope: Some(scope),
                history,
                token: Some(session.session_token.clone()),
                positions: Some(positions),
                number,
            };
            cases.extend(cases::for_actor(&actor, &rows));
        }
    }
    let fixture = Fixture {
        people: sessions.len(),
        offers: rows.len(),
        category_counts: counts,
        cases,
    };
    super::write_fixture(path, &fixture)?;
    println!(
        "v0.7 fixture: {} people, {} offers, {} cases",
        fixture.people,
        fixture.offers,
        fixture.cases.len()
    );
    Ok(())
}
