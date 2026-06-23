pub mod auth;
pub mod config;
pub mod crypto;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod ssh_metrics;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use config::AppConfig;
use system_pulse_db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<AppConfig>,
}

// Allows extractors written against `AppState` to also work if axum ever
// nests this inside a larger state struct (kept simple here — direct).
// impl FromRef<AppState> for AppState {
//     fn from_ref(state: &AppState) -> Self {
//         state.clone()
//     }
// }

pub fn build_router(db: Database, config: AppConfig) -> Router {
    let state = AppState {
        db: Arc::new(db),
        config: Arc::new(config),
    };

    let cors = if state.config.allowed_origins == vec!["*".to_string()] {
        CorsLayer::permissive()
    } else {
        use axum::http::HeaderValue;
        let origins: Vec<HeaderValue> = state
            .config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    };

    let auth_routes = Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))
        .route("/me", get(handlers::auth::me));

    let account_routes = Router::new()
        .route("/changepassword", post(handlers::account::change_password))
        .route("/changeemail", post(handlers::account::change_email))
        .route("/changelogin", post(handlers::account::change_login));

    let server_routes = Router::new()
        .route("/", get(handlers::servers::list).post(handlers::servers::create))
        .route(
            "/:id",
            get(handlers::servers::get)
                .put(handlers::servers::update)
                .delete(handlers::servers::delete),
        );

    let metrics_routes = Router::new()
        .route("/:server_id", get(handlers::metrics::get_recent))
        .route("/:server_id/latest", get(handlers::metrics::get_latest))
        .route("/:server_id/collect", post(handlers::metrics::collect_now));

    Router::new()
        .route("/health", get(handlers::health::health))
        .nest("/api/auth", auth_routes)
        .nest("/api/account", account_routes)
        .nest("/api/servers", server_routes)
        .nest("/api/metrics", metrics_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
