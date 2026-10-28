//! Category publishing, visibility and discovery regressions.

use super::*;
use haven_domain::offer::Category;

#[tokio::test]
async fn category_feed_intersects_facets_and_handles_empty_and_invalid_topics() {
    let (router, registry) = setup_test_app();
    let person = mock_user(UserId::generate(), AccountId::generate(), "category-maker");
    let session = create_auth_session(&registry.users_repo, person.clone());
    for (title, category, price, visible) in [
        ("Fresh pepper soup", Category::FoodGroceries, 500, true),
        (
            "Pepper growing textbook",
            Category::BooksLearning,
            500,
            true,
        ),
        ("Private pepper meal", Category::FoodGroceries, 500, false),
        ("Premium pepper meal", Category::FoodGroceries, 900, true),
    ] {
        let offer = registry
            .offers_repo
            .create(
                person.id,
                IdempotencyKey::new(Uuid::new_v4()),
                &CreateOffer {
                    title: title.to_string(),
                    description: "A useful local offer with clear details.".to_string(),
                    price: Price::new(price).unwrap(),
                    currency: CurrencyCode::Ngn,
                    category,
                },
            )
            .await
            .unwrap()
            .into_inner();
        if visible {
            registry
                .offers_repo
                .add_image(
                    offer.id,
                    person.id,
                    &haven_domain::image::ImagePublicId::parse(format!(
                        "offers/category/{}",
                        offer.id
                    ))
                    .unwrap(),
                    haven_domain::image::ImageDimensions::new(640, 480).unwrap(),
                )
                .await
                .unwrap();
        }
    }

    for (snapshot, uri, expected, excluded, noindex) in [
        (
            "all",
            "/offers",
            "Fresh pepper soup",
            "Private pepper meal",
            false,
        ),
        (
            "food",
            "/offers?category=food-groceries",
            "Fresh pepper soup",
            "Pepper growing textbook",
            false,
        ),
        (
            "filtered",
            "/offers?category=food-groceries&q=pepper&currency=NGN&max_price=600",
            "Fresh pepper soup",
            "Premium pepper meal",
            true,
        ),
        (
            "empty",
            "/offers?category=stays-accommodation",
            "No offers here yet",
            "Fresh pepper soup",
            false,
        ),
        (
            "no-matches",
            "/offers?category=stays-accommodation&q=pepper",
            "No offers match your filters",
            "Fresh pepper soup",
            true,
        ),
        (
            "invalid",
            "/offers?category=unknown-topic",
            "Choose a valid category",
            "Fresh pepper soup",
            true,
        ),
    ] {
        let response = router
            .handle(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert!(
            response
                .headers()
                .get_all("vary")
                .iter()
                .any(|v| v.to_str().unwrap().contains("Cookie"))
        );
        assert_eq!(response.headers().contains_key("x-robots-tag"), noindex);
        let body = body_to_string(response.into_body()).await;
        export_snapshot(&format!("{snapshot}.html"), &body);
        assert!(body.contains(expected), "{uri}");
        assert!(!body.contains(excluded), "{uri}");
        assert!(!body.contains("Private pepper meal"));
        let head = body.split("</head>").next().unwrap();
        assert_eq!(head.contains("rel=\"canonical\""), !noindex);
        let row = body
            .split("<nav class=\"chip-row\"")
            .skip(1)
            .find(|row| row.starts_with(" aria-label=\"Offer categories\""))
            .unwrap()
            .split("</nav>")
            .next()
            .unwrap();
        assert!(row.contains("aria-label=\"Offer categories\""));
        assert_eq!(row.matches("class=\"chip\"").count(), 12);
        assert_eq!(
            row.matches("aria-current=\"true\"").count(),
            usize::from(!uri.contains("unknown-topic"))
        );
        assert!(!body.contains("id=\"filter-category\""));
        for category in Category::ALL {
            assert!(row.contains(&format!("category={}", category.as_str())));
        }
    }
    let css = router
        .handle(
            Request::builder()
                .uri("/style.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    export_snapshot("style.css", &body_to_string(css.into_body()).await);
    let response = router
        .handle(
            Request::builder()
                .uri("/offers?category=food-groceries")
                .header("cookie", format!("sid={session}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert!(
        response
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("private, no-store")
    );
}

#[tokio::test]
async fn editing_requires_category_preserves_invalid_input_and_round_trips_changes() {
    let (router, registry) = setup_test_app();
    let person = mock_user(UserId::generate(), AccountId::generate(), "category-editor");
    let session = create_auth_session(&registry.users_repo, person.clone());
    let offer = registry
        .offers_repo
        .create(
            person.id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Fresh garden tomatoes".to_string(),
                description: "Fresh tomatoes offered by the basket.".to_string(),
                price: Price::new(500).unwrap(),
                currency: CurrencyCode::Ngn,
                category: Category::Other,
            },
        )
        .await
        .unwrap()
        .into_inner();
    for form in [
        "title=Retained+title",
        "title=Retained+title&category=unknown-topic",
        "title=Retained+title&category=",
    ] {
        let response = router
            .handle(
                Request::builder()
                    .method("POST")
                    .uri(format!("/profile/offers/{}/edit", offer.id))
                    .header("cookie", format!("sid={session}"))
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("origin", "http://localhost:3000")
                    .header("sec-fetch-site", "same-origin")
                    .body(Body::from(form))
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), http::StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("value=\"Retained title\""));
        assert!(body.contains("Choose a valid category."));
        assert!(body.contains("aria-invalid=\"true\""));
    }
    let response = router
        .handle(
            Request::builder()
                .method("POST")
                .uri(format!("/profile/offers/{}/edit", offer.id))
                .header("cookie", format!("sid={session}"))
                .header("content-type", "application/x-www-form-urlencoded")
                .header("origin", "http://localhost:3000")
                .header("sec-fetch-site", "same-origin")
                .body(Body::from("category=food-groceries"))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    let updated = registry
        .offers_repo
        .find_owned(offer.id, person.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.category, Category::FoodGroceries);
    assert_eq!(updated.title, offer.title);
    let response = router
        .handle(
            Request::builder()
                .uri(format!("/profile/offers/{}/edit", offer.id))
                .header("cookie", format!("sid={session}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("value=\"food-groceries\" selected=\"selected\""));
}

#[tokio::test]
async fn creation_rejects_missing_or_unknown_category_and_retains_fields() {
    let (router, registry) = setup_test_app();
    let person = mock_user(UserId::generate(), AccountId::generate(), "category-create");
    let session = create_auth_session(&registry.users_repo, person.clone());
    for category in ["", "unknown-topic"] {
        let (ct, body) = create_multipart_form(
            &[
                ("title", "Retained food offer"),
                (
                    "description",
                    "Retained description for a local food offer.",
                ),
                ("price", "500"),
                ("currency", "NGN"),
                ("category", category),
            ],
            &[],
        );
        let response = router
            .handle(
                Request::builder()
                    .method("POST")
                    .uri("/profile/offers/new")
                    .header("cookie", format!("sid={session}"))
                    .header("content-type", ct)
                    .header("origin", "http://localhost:3000")
                    .header("sec-fetch-site", "same-origin")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), http::StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("value=\"Retained food offer\""));
        assert!(body.contains("Choose a valid category."));
        assert!(body.contains("aria-invalid=\"true\""));
    }
    assert!(
        registry
            .offers_repo
            .find_by_user(person.id)
            .await
            .unwrap()
            .is_empty()
    );
}

// Opt-in, synthetic SSR snapshots for the final responsive review in CI.
fn export_snapshot(name: &str, body: &str) {
    if let Ok(directory) = std::env::var("HAVEN_PREVIEW_DIR")
        && !directory.is_empty()
    {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join(name), body).unwrap();
    }
}

#[tokio::test]
async fn malformed_price_cannot_silently_discard_a_category_filter() {
    let (router, _) = setup_test_app();
    for category in ["food-groceries", "unknown-topic"] {
        let response = router
            .handle(
                Request::builder()
                    .uri(format!(
                        "/offers?category={category}&max_price=not-a-number"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    }
}
