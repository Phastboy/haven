//! Authenticated collection routing, privacy, visibility and return intent.
use super::*;
use haven_domain::ports::SavedOfferRepository;

#[tokio::test]
async fn guests_sign_in_before_personal_retrieval_and_keep_their_facets() {
    let (router, _) = setup_test_app();
    for scope in ["saved", "inquired"] {
        let response = router.handle(Request::builder().uri(format!("/offers?scope={scope}&category=food-groceries&q=rice&currency=NGN&max_price=500"))
            .body(Body::empty()).unwrap()).await;
        assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
        assert!(
            response
                .headers()
                .get("cache-control")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("private, no-store")
        );
        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        let query = location.strip_prefix("/auth/sign-in?").unwrap();
        let target = url::form_urlencoded::parse(query.as_bytes())
            .find(|(key, _)| key == "return_to")
            .unwrap()
            .1
            .into_owned();
        assert_eq!(
            target,
            format!(
                "/offers?scope={scope}&category=food-groceries&q=rice&currency=NGN&max_price=500"
            )
        );
    }
}

#[tokio::test]
async fn personal_feed_uses_session_membership_and_private_noindex_responses() {
    let (router, registry) = setup_test_app();
    let actor = mock_user(UserId::generate(), AccountId::generate(), "personal-actor");
    let owner = mock_user(UserId::generate(), AccountId::generate(), "personal-owner");
    let token = create_auth_session(&registry.users_repo, actor.clone());
    let _owner_token = create_auth_session(&registry.users_repo, owner.clone());
    let mut ids = Vec::new();
    for title in ["Saved garden rice", "Unrelated garden rice"] {
        let offer = registry
            .offers_repo
            .create(
                owner.id,
                IdempotencyKey::new(Uuid::new_v4()),
                &CreateOffer {
                    category: haven_domain::offer::Category::FoodGroceries,
                    title: title.to_string(),
                    description: "Fresh rice offered locally by the bag.".to_string(),
                    price: Price::new(100).unwrap(),
                    currency: CurrencyCode::Ngn,
                },
            )
            .await
            .unwrap()
            .into_inner();
        registry
            .offers_repo
            .add_image(
                offer.id,
                owner.id,
                &haven_domain::image::ImagePublicId::parse(format!("offers/personal/{}", offer.id))
                    .unwrap(),
                haven_domain::image::ImageDimensions::new(640, 480).unwrap(),
            )
            .await
            .unwrap();
        ids.push(offer.id);
    }
    registry.saved_repo.save(actor.id, ids[0]).await.unwrap();
    registry.saved_repo.save(owner.id, ids[1]).await.unwrap();
    registry
        .offers_repo
        .inquired
        .lock()
        .unwrap()
        .insert((actor.id, ids[0]), Utc::now());
    for scope in ["saved", "inquired"] {
        let response = router
            .handle(
                Request::builder()
                    .uri(format!(
                        "/offers?scope={scope}&category=food-groceries&user_id={}",
                        owner.id
                    ))
                    .header("cookie", format!("sid={token}"))
                    .header("if-none-match", "guest-tag")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert!(
            response
                .headers()
                .get("cache-control")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("private, no-store")
        );
        assert!(
            response
                .headers()
                .get("x-robots-tag")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("noindex")
        );
        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("Saved garden rice"));
        assert!(!body.contains("Unrelated garden rice"));
        assert!(
            !body
                .split("</head>")
                .next()
                .unwrap()
                .contains("rel=\"canonical\"")
        );
        assert!(body.contains(&format!("type=\"hidden\" name=\"scope\" value=\"{scope}\"")));
    }
    registry.saved_repo.unsave(actor.id, ids[0]).await.unwrap();
    let response = router
        .handle(
            Request::builder()
                .uri("/offers?scope=saved")
                .header("cookie", format!("sid={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert!(
        body_to_string(response.into_body())
            .await
            .contains("No saved offers yet")
    );
}
