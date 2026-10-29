//! State and concurrency regressions for the indexed unread predicate.

use super::*;

async fn people(pool: &PgPool) -> [UserId; 3] {
    let people = std::array::from_fn(|_| UserId::from_uuid(Uuid::new_v4()));
    for person in people {
        let account = Uuid::new_v4();
        sqlx::query("INSERT INTO account (id,email) VALUES ($1,$2)")
            .bind(account)
            .bind(format!("{account}@example.com"))
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO \"user\" (id,account_id,display_name,handle) VALUES ($1,$2,'Unread test',$3)")
            .bind(person.as_uuid()).bind(account).bind(format!("unread-{}", &account.to_string()[..16]))
            .execute(pool).await.unwrap();
    }
    people
}

fn message(sender_id: UserId) -> NewMessage {
    NewMessage {
        sender_id,
        body: MessageBody::parse("Unread state regression").unwrap(),
        offer_id: None,
    }
}

async fn assert_consistent(pool: &PgPool) {
    let mismatches = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM conversation_participant cp
        JOIN conversation c ON c.id=cp.conversation_id
        WHERE cp.is_unread <> COALESCE(
            cp.last_read_at < c.last_message_at AND (
                SELECT sender_id FROM message WHERE conversation_id=c.id
                ORDER BY created_at DESC,id DESC LIMIT 1
            ) <> cp.user_id, FALSE)"#,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(mismatches, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn indexed_unread_preserves_membership_and_latest_sender(pool: PgPool) {
    let [sender, recipient, outsider] = people(&pool).await;
    let repo = PostgresConversationRepository::new(pool.clone());
    let conversation = repo
        .get_or_create(ConversationPair::new(sender, recipient).unwrap())
        .await
        .unwrap();
    assert!(!repo.unread_exists(sender).await.unwrap());
    assert!(!repo.unread_exists(recipient).await.unwrap());
    assert!(matches!(
        repo.mark_read(conversation.id, outsider).await,
        Err(RepoError::NotFound { .. })
    ));
    assert!(matches!(
        repo.append(conversation.id, message(outsider)).await,
        Err(RepoError::NotFound { .. })
    ));
    let participants = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM conversation_participant WHERE conversation_id=$1",
    )
    .bind(conversation.id.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(participants, 2);

    repo.append(conversation.id, message(sender)).await.unwrap();
    assert!(repo.unread_exists(recipient).await.unwrap());
    assert!(!repo.unread_exists(sender).await.unwrap());
    repo.append(conversation.id, message(recipient))
        .await
        .unwrap();
    assert!(!repo.unread_exists(recipient).await.unwrap());
    assert!(repo.unread_exists(sender).await.unwrap());
    repo.mark_read(conversation.id, sender).await.unwrap();
    repo.mark_read(conversation.id, sender).await.unwrap();
    assert!(!repo.unread_exists(sender).await.unwrap());
    assert_consistent(&pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn concurrent_sends_and_reads_preserve_unread_state(pool: PgPool) {
    let [first, second, _] = people(&pool).await;
    let repo = PostgresConversationRepository::new(pool.clone());
    let conversation = repo
        .get_or_create(ConversationPair::new(first, second).unwrap())
        .await
        .unwrap();
    for _ in 0..12 {
        let (send_first, read_second, send_second, read_first) = tokio::join!(
            repo.append(conversation.id, message(first)),
            repo.mark_read(conversation.id, second),
            repo.append(conversation.id, message(second)),
            repo.mark_read(conversation.id, first),
        );
        send_first.unwrap();
        read_second.unwrap();
        send_second.unwrap();
        read_first.unwrap();
        assert_consistent(&pool).await;
    }
    repo.mark_read(conversation.id, first).await.unwrap();
    repo.mark_read(conversation.id, second).await.unwrap();
    assert!(!repo.unread_exists(first).await.unwrap());
    assert!(!repo.unread_exists(second).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn bulk_fixture_timestamp_refreshes_unread_state(pool: PgPool) {
    let [sender, recipient, _] = people(&pool).await;
    let repo = PostgresConversationRepository::new(pool.clone());
    let conversation = repo
        .get_or_create(ConversationPair::new(sender, recipient).unwrap())
        .await
        .unwrap();
    sqlx::query(
        "UPDATE conversation_participant SET last_read_at='2026-01-01' WHERE conversation_id=$1",
    )
    .bind(conversation.id.as_uuid())
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO message (conversation_id,sender_id,body,created_at) VALUES ($1,$2,'Bulk fixture','2026-02-01')")
        .bind(conversation.id.as_uuid()).bind(sender.as_uuid()).execute(&pool).await.unwrap();
    sqlx::query("UPDATE conversation SET last_message_at='2026-02-01' WHERE id=$1")
        .bind(conversation.id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    assert!(repo.unread_exists(recipient).await.unwrap());
    assert!(!repo.unread_exists(sender).await.unwrap());
    assert_consistent(&pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn dense_read_history_uses_partial_unread_index(pool: PgPool) {
    let [person, _, _] = people(&pool).await;
    sqlx::query(
        "INSERT INTO account (id,email) SELECT gen_random_uuid(), 'dense-' || n || '@example.com' FROM generate_series(1,2048) n",
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO \"user\" (id,account_id,display_name,handle) SELECT gen_random_uuid(),id,'Dense history','dense-' || substr(replace(id::text,'-',''),1,24) FROM account WHERE email LIKE 'dense-%'",
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO conversation (user_low,user_high) SELECT LEAST($1::uuid,id),GREATEST($1::uuid,id) FROM \"user\" WHERE handle LIKE 'dense-%'",
    ).bind(person.as_uuid()).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO conversation_participant (conversation_id,user_id) SELECT id,$1 FROM conversation WHERE user_low=$1 OR user_high=$1",
    ).bind(person.as_uuid()).execute(&pool).await.unwrap();
    sqlx::query("ANALYZE conversation_participant")
        .execute(&pool)
        .await
        .unwrap();
    let plan = sqlx::query_scalar::<_, String>(
        "EXPLAIN (ANALYZE, BUFFERS) SELECT EXISTS (SELECT 1 FROM conversation_participant WHERE user_id=$1 AND is_unread)",
    ).bind(person.as_uuid()).fetch_all(&pool).await.unwrap().join("\n");
    assert!(plan.contains("idx_participant_unread_user"), "{plan}");
    assert!(!plan.contains("Seq Scan"), "{plan}");
    assert!(!plan.contains("idx_message_thread"), "{plan}");
}

#[sqlx::test(migrations = false)]
async fn migration_backfills_populated_and_empty_conversations(pool: PgPool) {
    let migrator = sqlx::migrate!("../../migrations");
    for migration in migrator.iter().filter(|migration| migration.version < 19) {
        let mut tx = pool.begin().await.unwrap();
        sqlx::raw_sql(migration.sql.clone())
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
    let [sender, recipient, third] = people(&pool).await;
    let repo = PostgresConversationRepository::new(pool.clone());
    let conversation = repo
        .get_or_create(ConversationPair::new(sender, recipient).unwrap())
        .await
        .unwrap();
    repo.get_or_create(ConversationPair::new(sender, third).unwrap())
        .await
        .unwrap();
    repo.append(conversation.id, message(sender)).await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../../../../migrations/019_indexed_unread_state.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert!(repo.unread_exists(recipient).await.unwrap());
    assert!(!repo.unread_exists(sender).await.unwrap());
    assert!(!repo.unread_exists(third).await.unwrap());
    assert_consistent(&pool).await;
}
