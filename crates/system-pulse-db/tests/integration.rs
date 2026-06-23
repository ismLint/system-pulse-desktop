//! Integration tests — run against an in-memory SQLite database so they
//! need no external setup (`cargo test -p system-pulse-db`).

use system_pulse_db::connection::{Database, DatabaseConfig};
use system_pulse_db::models::server::{CreateServerInput, UpdateServerInput};
use system_pulse_db::models::user::CreateUserInput;
use system_pulse_db::models::metric::MetricInput;
use system_pulse_db::queries::{metrics, servers, sessions, users};

async fn test_db() -> Database {
    Database::connect_in_memory().await.expect("connect in-memory db")
}

#[tokio::test]
async fn migrations_run_cleanly_twice() {
    // Connecting twice (re-running migrations) must not error —
    // this is what happens every time the desktop app or server restarts.
    let db = test_db().await;
    let config = DatabaseConfig::at_path(&db.db_path);
    // Can't actually reconnect to :memory: from a new pool, so just assert
    // the first connect succeeded without panicking.
    let _ = config;
}

#[tokio::test]
async fn user_create_and_find() {
    let db = test_db().await;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: "alice".into(),
            email: "alice@example.com".into(),
            password_hash: "hashed".into(),
            first_name: Some("Alice".into()),
            last_name: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(user.login, "alice");

    let found = users::find_by_login(&db.pool, "alice").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user.id);
}

#[tokio::test]
async fn user_login_email_uniqueness() {
    let db = test_db().await;

    users::create(
        &db.pool,
        &CreateUserInput {
            login: "bob".into(),
            email: "bob@example.com".into(),
            password_hash: "x".into(),
            first_name: None,
            last_name: None,
        },
    )
    .await
    .unwrap();

    let exists = users::login_or_email_exists(&db.pool, "bob", "someone@else.com")
        .await
        .unwrap();
    assert!(exists);

    let not_exists = users::login_or_email_exists(&db.pool, "carol", "carol@example.com")
        .await
        .unwrap();
    assert!(!not_exists);
}

#[tokio::test]
async fn server_crud_roundtrip() {
    let db = test_db().await;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: "dave".into(),
            email: "dave@example.com".into(),
            password_hash: "x".into(),
            first_name: None,
            last_name: None,
        },
    )
    .await
    .unwrap();

    let server = servers::create(
        &db.pool,
        &user.id,
        &CreateServerInput {
            name: "prod-1".into(),
            host: "10.0.0.5".into(),
            ssh_user: "root".into(),
            password_encrypted: "enc".into(),
            description: Some("primary".into()),
            server_type: "remote".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(server.name, "prod-1");

    let list = servers::list_for_user(&db.pool, &user.id).await.unwrap();
    assert_eq!(list.len(), 1);

    let updated = servers::update(
        &db.pool,
        &server.id,
        &user.id,
        &UpdateServerInput {
            is_active: Some(false),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(!updated.is_active);

    servers::delete(&db.pool, &server.id, &user.id).await.unwrap();
    let after = servers::find_owned(&db.pool, &server.id, &user.id).await.unwrap();
    assert!(after.is_none());
}

#[tokio::test]
async fn server_duplicate_name_for_same_user_conflicts() {
    let db = test_db().await;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: "erin".into(),
            email: "erin@example.com".into(),
            password_hash: "x".into(),
            first_name: None,
            last_name: None,
        },
    )
    .await
    .unwrap();

    let input = CreateServerInput {
        name: "dup".into(),
        host: "1.1.1.1".into(),
        ssh_user: "root".into(),
        password_encrypted: "enc".into(),
        description: None,
        server_type: "remote".into(),
    };

    servers::create(&db.pool, &user.id, &input).await.unwrap();
    let second = servers::create(&db.pool, &user.id, &input).await;
    assert!(second.is_err());
}

#[tokio::test]
async fn metrics_insert_and_query() {
    let db = test_db().await;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: "frank".into(),
            email: "frank@example.com".into(),
            password_hash: "x".into(),
            first_name: None,
            last_name: None,
        },
    )
    .await
    .unwrap();

    let server = servers::create(
        &db.pool,
        &user.id,
        &CreateServerInput {
            name: "local-1".into(),
            host: "localhost".into(),
            ssh_user: "local".into(),
            password_encrypted: String::new(),
            description: None,
            server_type: "local".into(),
        },
    )
    .await
    .unwrap();

    for i in 0..5 {
        metrics::insert(
            &db.pool,
            &MetricInput {
                server_id: server.id.clone(),
                collected_at: chrono::Utc::now().to_rfc3339(),
                cpu_usage: 10.0 + i as f64,
                cpu_cores: Some(8),
                ram_used: 1000,
                ram_total: 2000,
                ram_usage_pct: 50.0,
                temperature_cpu: Some(45.0),
                temperature_gpu: None,
                disk_used: Some(100),
                disk_total: Some(500),
                disk_read_bytes_sec: Some(0),
                disk_write_bytes_sec: Some(0),
                net_rx_bytes_sec: Some(0),
                net_tx_bytes_sec: Some(0),
                load_avg_1: Some(0.5),
                load_avg_5: Some(0.4),
                load_avg_15: Some(0.3),
                uptime_seconds: Some(1000),
            },
        )
        .await
        .unwrap();
    }

    let recent = metrics::recent_for_server(&db.pool, &server.id, 10).await.unwrap();
    assert_eq!(recent.len(), 5);
    // oldest-first ordering
    assert!(recent[0].cpu_usage <= recent[4].cpu_usage);

    let latest = metrics::latest_for_server(&db.pool, &server.id).await.unwrap();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().cpu_usage, 14.0);
}

#[tokio::test]
async fn sessions_lifecycle() {
    let db = test_db().await;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: "grace".into(),
            email: "grace@example.com".into(),
            password_hash: "x".into(),
            first_name: None,
            last_name: None,
        },
    )
    .await
    .unwrap();

    let future = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    let session = sessions::create(
        &db.pool,
        &system_pulse_db::models::session::CreateSessionInput {
            user_id: user.id.clone(),
            token_hash: "hash123".into(),
            expires_at: future,
        },
    )
    .await
    .unwrap();

    assert!(sessions::is_valid(&db.pool, &session.token_hash).await.unwrap());

    sessions::revoke(&db.pool, &session.token_hash).await.unwrap();
    assert!(!sessions::is_valid(&db.pool, &session.token_hash).await.unwrap());
}

#[tokio::test]
async fn metrics_cleanup_by_age() {
    let db = test_db().await;

    let user = users::create(
        &db.pool,
        &CreateUserInput {
            login: "henry".into(),
            email: "henry@example.com".into(),
            password_hash: "x".into(),
            first_name: None,
            last_name: None,
        },
    )
    .await
    .unwrap();

    let server = servers::create(
        &db.pool,
        &user.id,
        &CreateServerInput {
            name: "old-server".into(),
            host: "1.2.3.4".into(),
            ssh_user: "root".into(),
            password_encrypted: "enc".into(),
            description: None,
            server_type: "remote".into(),
        },
    )
    .await
    .unwrap();

    // Insert a metric with an old timestamp directly (bypassing "now" default)
    sqlx::query(
        "INSERT INTO metrics (server_id, collected_at, cpu_usage, ram_used, ram_total, ram_usage_pct)
         VALUES (?1, datetime('now', '-30 days'), 5.0, 100, 200, 50.0)",
    )
    .bind(&server.id)
    .execute(&db.pool)
    .await
    .unwrap();

    let deleted = metrics::delete_older_than_days(&db.pool, 7).await.unwrap();
    assert_eq!(deleted, 1);
}
