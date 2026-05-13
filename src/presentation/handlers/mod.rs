use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::application::dtos::CreateOrderInput;
use crate::AppState;
use crate::domain::errors::DomainError;

pub async fn create_order(
    State(state): State<AppState>,
    Json(input): Json<CreateOrderInput>,
) -> (StatusCode, Json<Value>) {
    match state.create_order.execute(input).await {
        Ok(output) => (StatusCode::CREATED, Json(json!(output))),
        Err(DomainError::EmptyOrder | DomainError::InvalidQuantity) => {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": "validation failed"})))
        }
        Err(e) => {
            tracing::error!(?e, "unexpected error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "internal error"})))
        }
    }
}

pub async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}
