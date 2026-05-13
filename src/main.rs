mod domain;
mod application;
mod infrastructure;
mod presentation;

use std::sync::Arc;
use axum::{routing::{get, post}, Router};
use application::use_cases::create_order::CreateOrderUseCase;

#[derive(Clone)]
pub struct AppState {
    pub create_order: Arc<CreateOrderUseCase>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let _pool = infrastructure::database::create_pool(&database_url).await.expect("Failed to create pool");

    // TODO: Wire up repository implementation
    // let repo = Arc::new(PgOrderRepository::new(pool));
    // let create_order = Arc::new(CreateOrderUseCase::new(repo));

    let app = Router::new()
        .route("/health", get(presentation::handlers::health));
        // .route("/api/v1/orders", post(presentation::handlers::create_order))
        // .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    tracing::info!("Server started on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
